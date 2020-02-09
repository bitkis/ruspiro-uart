# Changelog
## :pizza: v0.4.0
  - ### :bulb: Features
    - Uart0 allows now to register a "callback" that will executed once an Uart0 related interrupt occurs.
    This ensures the whole low-level interrupt handling e.g. acknowledgement happens inside the ``uart`` crate.
    - This create refernces now the ``ruspiro-core`` crate and introduces specific error types that increase
    error handling usability
  - ### :detective: Fixes
    Bug-fixes
  - ### :wrench: Maintenance
    - update to use latest ``ruspiro-register`` version
    - update to use latest ``ruspiro-gpio`` version
    - feature ``ruspiro_pi3`` is no longer active by default

  - ### :book: Documentation
    Updates in documentation (inline, readme...)
