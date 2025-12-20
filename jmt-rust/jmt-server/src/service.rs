//! WebSocket service for client-server communication

use std::path::PathBuf;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use prost::Message as ProstMessage;

use jmt_proto::{Command, Response, command, response};
use crate::file_ops;
use crate::ServerConfig;

/// Run the WebSocket server
pub async fn run_server(config: ServerConfig, mut shutdown: broadcast::Receiver<()>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&config.addr).await?;
    info!("JMT Server listening on {}", config.addr);

    // Ensure project directory exists
    file_ops::ensure_project_dir(&PathBuf::from(&config.project_path))?;

    let config = Arc::new(config);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        info!("New connection from {}", addr);
                        let config = Arc::clone(&config);
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, config).await {
                                error!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("Server shutting down");
                break;
            }
        }
    }

    Ok(())
}

/// Handle a single WebSocket connection
async fn handle_connection(stream: TcpStream, config: Arc<ServerConfig>) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                match Command::decode(data.as_slice()) {
                    Ok(cmd) => {
                        let response = handle_command(cmd, &config).await;
                        let mut buf = Vec::new();
                        response.encode(&mut buf)?;
                        write.send(Message::Binary(buf.into())).await?;
                    }
                    Err(e) => {
                        warn!("Failed to decode command: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client closed connection");
                break;
            }
            Ok(Message::Ping(data)) => {
                write.send(Message::Pong(data)).await?;
            }
            Ok(_) => {
                // Ignore text and other message types
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Handle a command and return a response
async fn handle_command(cmd: Command, config: &ServerConfig) -> Response {
    let request_id = cmd.request_id;

    let response_type = match cmd.command {
        Some(command::Command::NewDiagram(req)) => {
            let diagram = file_ops::new_diagram(&req.name);
            // Convert to proto diagram
            let proto_diagram = diagram_to_proto(&diagram);
            response::Response::NewDiagram(jmt_proto::NewDiagramResponse {
                diagram: Some(proto_diagram),
            })
        }
        Some(command::Command::OpenDiagram(req)) => {
            let path = PathBuf::from(&req.path);
            match file_ops::load_diagram(&path) {
                Ok(diagram) => {
                    let proto_diagram = diagram_to_proto(&diagram);
                    response::Response::OpenDiagram(jmt_proto::OpenDiagramResponse {
                        result: Some(jmt_proto::open_diagram_response::Result::Diagram(proto_diagram)),
                    })
                }
                Err(e) => {
                    response::Response::OpenDiagram(jmt_proto::OpenDiagramResponse {
                        result: Some(jmt_proto::open_diagram_response::Result::Error(e.to_string())),
                    })
                }
            }
        }
        Some(command::Command::SaveDiagram(req)) => {
            if let Some(proto_diagram) = req.diagram {
                let diagram = diagram_from_proto(&proto_diagram);
                let path = req.path.unwrap_or_else(|| {
                    format!("{}/{}.json", config.project_path, diagram.settings.name)
                });
                let path = PathBuf::from(&path);

                match file_ops::save_diagram(&diagram, &path) {
                    Ok(()) => {
                        response::Response::SaveDiagram(jmt_proto::SaveDiagramResponse {
                            success: true,
                            error: None,
                            saved_path: Some(path.display().to_string()),
                        })
                    }
                    Err(e) => {
                        response::Response::SaveDiagram(jmt_proto::SaveDiagramResponse {
                            success: false,
                            error: Some(e.to_string()),
                            saved_path: None,
                        })
                    }
                }
            } else {
                response::Response::Error("No diagram provided".to_string())
            }
        }
        Some(command::Command::ListFiles(req)) => {
            let dir = PathBuf::from(&req.directory);
            let ext = if req.extension_filter.is_empty() {
                None
            } else {
                Some(req.extension_filter.as_str())
            };

            match file_ops::list_files(&dir, ext) {
                Ok(files) => {
                    let proto_files: Vec<jmt_proto::FileInfo> = files
                        .into_iter()
                        .map(|f| jmt_proto::FileInfo {
                            name: f.name,
                            path: f.path.display().to_string(),
                            modified_time: f.modified_time as i64,
                        })
                        .collect();
                    response::Response::ListFiles(jmt_proto::ListFilesResponse { files: proto_files })
                }
                Err(e) => {
                    response::Response::Error(e.to_string())
                }
            }
        }
        Some(command::Command::GetProjectPath(_)) => {
            response::Response::GetProjectPath(jmt_proto::GetProjectPathResponse {
                path: config.project_path.clone(),
            })
        }
        None => {
            response::Response::Error("Empty command".to_string())
        }
    };

    Response {
        request_id,
        response: Some(response_type),
    }
}

// Conversion functions between core and proto types
// These would be more complete in a real implementation

fn diagram_to_proto(diagram: &jmt_core::Diagram) -> jmt_proto::Diagram {
    // Simplified conversion - in a real app this would be complete
    jmt_proto::Diagram {
        id: diagram.id.to_string(),
        settings: Some(jmt_proto::DiagramSettings {
            name: diagram.settings.name.clone(),
            file_path: diagram.settings.file_path.clone(),
            state_color: Some(jmt_proto::Color {
                r: diagram.settings.state_color.r as u32,
                g: diagram.settings.state_color.g as u32,
                b: diagram.settings.state_color.b as u32,
                a: diagram.settings.state_color.a as u32,
            }),
            corner_rounding: diagram.settings.corner_rounding,
            stub_length: diagram.settings.stub_length,
            arrow_width: diagram.settings.arrow_width,
            arrow_height: diagram.settings.arrow_height,
            corner_size: diagram.settings.corner_size,
            pseudo_corner_size: diagram.settings.pseudo_corner_size,
            default_state_width: diagram.settings.default_state_width,
            default_state_height: diagram.settings.default_state_height,
            default_pseudo_size: diagram.settings.default_pseudo_size,
        }),
        root_state: Some(state_to_proto(&diagram.root_state)),
        nodes: diagram.nodes().iter().map(node_to_proto).collect(),
        connections: diagram.connections().iter().map(connection_to_proto).collect(),
    }
}

fn state_to_proto(state: &jmt_core::node::State) -> jmt_proto::StateNode {
    jmt_proto::StateNode {
        id: state.id.to_string(),
        name: state.name.clone(),
        bounds: Some(jmt_proto::Rect {
            x1: state.bounds.x1,
            y1: state.bounds.y1,
            x2: state.bounds.x2,
            y2: state.bounds.y2,
        }),
        fill_color: state.fill_color.map(|c| jmt_proto::Color {
            r: c.r as u32,
            g: c.g as u32,
            b: c.b as u32,
            a: c.a as u32,
        }),
        parent_region_id: state.parent_region_id.map(|id| id.to_string()),
        entry_activity: state.entry_activity.clone(),
        exit_activity: state.exit_activity.clone(),
        do_activity: state.do_activity.clone(),
        regions: state.regions.iter().map(region_to_proto).collect(),
    }
}

fn region_to_proto(region: &jmt_core::node::Region) -> jmt_proto::Region {
    jmt_proto::Region {
        id: region.id.to_string(),
        name: region.name.clone(),
        bounds: Some(jmt_proto::Rect {
            x1: region.bounds.x1,
            y1: region.bounds.y1,
            x2: region.bounds.x2,
            y2: region.bounds.y2,
        }),
        child_node_ids: region.children.iter().map(|id| id.to_string()).collect(),
        is_horizontal: region.is_horizontal,
    }
}

fn node_to_proto(node: &jmt_core::Node) -> jmt_proto::Node {
    match node {
        jmt_core::Node::State(state) => jmt_proto::Node {
            node: Some(jmt_proto::node::Node::State(state_to_proto(state))),
        },
        jmt_core::Node::Pseudo(pseudo) => jmt_proto::Node {
            node: Some(jmt_proto::node::Node::Pseudo(jmt_proto::PseudoStateNode {
                id: pseudo.id.to_string(),
                name: pseudo.name.clone(),
                kind: pseudo_kind_to_proto(pseudo.kind) as i32,
                bounds: Some(jmt_proto::Rect {
                    x1: pseudo.bounds.x1,
                    y1: pseudo.bounds.y1,
                    x2: pseudo.bounds.x2,
                    y2: pseudo.bounds.y2,
                }),
                fill_color: Some(jmt_proto::Color {
                    r: pseudo.fill_color.r as u32,
                    g: pseudo.fill_color.g as u32,
                    b: pseudo.fill_color.b as u32,
                    a: pseudo.fill_color.a as u32,
                }),
                parent_region_id: pseudo.parent_region_id.map(|id| id.to_string()),
            })),
        },
    }
}

fn pseudo_kind_to_proto(kind: jmt_core::node::PseudoStateKind) -> jmt_proto::NodeType {
    match kind {
        jmt_core::node::PseudoStateKind::Initial => jmt_proto::NodeType::Initial,
        jmt_core::node::PseudoStateKind::Final => jmt_proto::NodeType::Final,
        jmt_core::node::PseudoStateKind::Choice => jmt_proto::NodeType::Choice,
        jmt_core::node::PseudoStateKind::Fork => jmt_proto::NodeType::Fork,
        jmt_core::node::PseudoStateKind::Join => jmt_proto::NodeType::Join,
        jmt_core::node::PseudoStateKind::Junction => jmt_proto::NodeType::Junction,
    }
}

fn connection_to_proto(conn: &jmt_core::Connection) -> jmt_proto::Connection {
    jmt_proto::Connection {
        id: conn.id.to_string(),
        name: conn.name.clone(),
        source_id: conn.source_id.to_string(),
        target_id: conn.target_id.to_string(),
        source_side: side_to_proto(conn.source_side) as i32,
        target_side: side_to_proto(conn.target_side) as i32,
        source_offset: conn.source_offset,
        target_offset: conn.target_offset,
        event: conn.event.clone(),
        guard: conn.guard.clone(),
        action: conn.action.clone(),
    }
}

fn side_to_proto(side: jmt_core::Side) -> jmt_proto::Side {
    match side {
        jmt_core::Side::None => jmt_proto::Side::None,
        jmt_core::Side::Top => jmt_proto::Side::Top,
        jmt_core::Side::Bottom => jmt_proto::Side::Bottom,
        jmt_core::Side::Left => jmt_proto::Side::Left,
        jmt_core::Side::Right => jmt_proto::Side::Right,
    }
}

// Reverse conversions (proto to core) - simplified
fn diagram_from_proto(proto: &jmt_proto::Diagram) -> jmt_core::Diagram {
    // This is a simplified implementation - a full implementation would
    // convert all fields properly
    let mut diagram = jmt_core::Diagram::new(
        proto.settings.as_ref().map(|s| s.name.as_str()).unwrap_or("Untitled")
    );

    if let Some(settings) = &proto.settings {
        diagram.settings.file_path = settings.file_path.clone();
    }

    diagram
}
