<!-- AIDA Generated: v1.0.0 | checksum:b182b483 | DO NOT EDIT DIRECTLY -->
<!-- To customize: copy to .claude/skills/custom/ and modify there -->

# Evaluate Requirement

Evaluate a requirement's quality using AI analysis.

## Usage

```
/aida-evaluate <SPEC-ID>
```

## Instructions

Follow the workflow in `.claude/skills/aida-evaluate.md`:

1. Load the requirement from the database
2. Assess clarity, testability, completeness, and consistency
3. Generate quality score (1-10) with detailed feedback
4. Offer follow-up actions: improve, split, or accept
