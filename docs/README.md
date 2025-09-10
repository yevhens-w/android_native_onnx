# Documentation

This directory contains architectural documentation and UML diagrams for the Shared Core library.

## Files

- `architecture.md` - Comprehensive UML diagrams including:
  - Class Diagram - Shows the structure and relationships between all components
  - Sequence Diagram - Illustrates the backend selection process
  - Component Diagram - Shows the modular architecture and dependencies  
  - State Diagram - Documents the backend lifecycle states
  - Activity Diagram - Maps the data processing workflow

## Viewing the Diagrams

The UML diagrams are written in Mermaid format and can be viewed in:
- GitHub (renders Mermaid automatically)
- VS Code with Mermaid extensions
- Online Mermaid editors (https://mermaid.live)
- Any Markdown viewer that supports Mermaid

## Architecture Overview

The Shared Core library follows a modular, trait-based architecture:

1. **Runner System** - Central orchestration of backend selection and processing
2. **ComputeRunner Trait** - Unified interface for all backend implementations
3. **Backend Implementations** - CPU, ARM NEON, and GPU-specific processing
4. **Hardware Detection** - Runtime detection of available compute capabilities
5. **UniFFI Bindings** - Cross-platform interface generation for mobile apps