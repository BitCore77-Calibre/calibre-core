# Calibre Protocol — Whitepaper

**Version:** 0.6.0-testnet
**Status:** Public draft — not yet audited, not for production use.

## Documents

- **[Full whitepaper (Part 1)](./calibre-whitepaper.md)** — Abstract, Problem, Architecture
- **[Part 2](./calibre-whitepaper-part2.md)** — Cryptographic Specification, Verification Paths
- **[Part 3](./calibre-whitepaper-part3.md)** — Fast-Path Mempool, Interoperability, Tokenomics
- **[Part 4](./calibre-whitepaper-part4.md)** — Status, Roadmap, References
- **[Executive Summary](./executive-summary.md)** — one-page overview for non-technical readers

## Rendering to PDF

The whitepaper is written in Markdown. To generate a professional PDF:

    pandoc calibre-whitepaper.md \
      -o calibre-whitepaper.pdf \
      --pdf-engine=xelatex \
      --toc \
      --number-sections \
      --highlight-style=tango

To render Mermaid diagrams inside the PDF, pre-render them with the
Mermaid CLI (mmdc) or use the pandoc-mermaid filter.

## Citing this document

    @misc{calibre2026whitepaper,
      title  = {Calibre Protocol: A Quantum-Resistant Layer-1 Cryptoeconomy},
      author = {Calibre Protocol Contributors},
      year   = {2026},
      note   = {Version 0.6.0-testnet},
      url    = {https://github.com/calibre-protocol/calibre-template}
    }
