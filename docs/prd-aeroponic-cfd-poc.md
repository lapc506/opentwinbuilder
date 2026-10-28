# Product Requirements Document / Software Requirements Specification
## Aeroponic CAD-to-CFD Proof of Concept

Version 0.1  
Prepared by: CAD Integration Hub  
Repository: `lapc506/open-polygon-3D-to-CAD-bridge`  
Date: 2026-09-07  

> This document follows the structure and requirements discipline of the [SRS-Template](https://github.com/jam01/SRS-Template), which is aligned with IEEE 830 and ISO/IEC/IEEE 29148. It is adapted for a proof of concept: requirements define **what must be demonstrated**; implementation choices remain subject to architecture and engineering decisions.

## Revision History

| Name | Date | Reason For Changes | Version |
|------|------|--------------------|---------|
| CAD Integration Hub | 2026-09-07 | Initial aeroponic CFD PoC definition | 0.1 |

# 1. Introduction

## 1.1 Document Purpose

This PRD/SRS defines the proof of concept for transforming a Blender-modeled, substrate-free aeroponic microgreenhouse into a semantic, parametric engineering model and using that model to drive hydraulic and CFD analysis of nutrient-solution mist delivery to suspended crop roots.

The document is intended for product, architecture, software, CFD, CAD, agronomy, and QA work. It establishes a verifiable vertical slice from Blender geometry through a CAD Integration Hub Common Intermediate Model (CIM), simulation inputs, CFD results, and parameter updates.

## 1.2 Product Scope

The PoC will demonstrate a workflow in which a user creates or imports a microgreenhouse/aeroponic chamber in Blender, including root zones and real-world mister components, and the bridge reconstructs those objects as semantic CIM entities. The resulting model shall expose engineering parameters such as mister location, orientation, pressure, flow, nozzle definition, droplet distribution, piping, chamber dimensions, and root-zone geometry.

The PoC shall generate a simulation-ready model for hydraulic analysis and droplet transport CFD, run a supported solver workflow, visualize or expose root-zone delivery results, and permit parameter changes followed by regeneration and re-analysis.

The PoC does **not** attempt to prove a universal pressure-to-droplet-size relationship, nor does it initially model detailed atomization/breakup inside the nozzle. Droplet distributions are treated as measured, manufacturer-provided, or explicitly calibrated injection data.

## 1.3 Definitions, Acronyms, and Abbreviations

| Term | Definition |
|------|------------|
| CAD | Computer-Aided Design |
| CFD | Computational Fluid Dynamics |
| CIM | CAD Integration Hub Common Intermediate Model |
| PoC | Proof of Concept |
| Mister | Irrigation atomizer/mister delivering liquid as droplets |
| Root zone | Spatial region occupied by or surrounding suspended crop roots |
| Droplet population | Statistical representation of injected droplet sizes and properties |
| Injection | CFD representation of droplets entering the fluid domain from a mister |
| Parametric model | Model whose geometry and physical behavior are controlled by explicit parameters |
| SRS | Software Requirements Specification |
| PRD | Product Requirements Document |

## 1.4 References

### Normative / process references

1. `jam01/SRS-Template` — Markdown Software Requirements Specification template aligned with IEEE 830 and ISO/IEC/IEEE 29148: https://github.com/jam01/SRS-Template
2. `lapc506/open-polygon-3D-to-CAD-bridge` — target repository for this PoC: https://github.com/lapc506/open-polygon-3D-to-CAD-bridge

### Informative engineering references

3. Hunter Agriculture / Senninger — WinSIPP 3D sprinkler/micro-irrigation analysis tooling: https://agriculture.hunterirrigation.com/irrigation-product/winsipptm3
4. Hunter Agriculture / Senninger — SennREG pressure-regulator utility: https://agriculture.hunterirrigation.com/irrigation-product/sennregtm
5. Drip Depot — Nursery and greenhouse irrigation reference: https://help.dripdepot.com/support/solutions/articles/11000097617-nursery-and-greenhouse-irrigation
6. Hunter Agriculture / Senninger — Mister product information: https://agriculture.hunterirrigation.com/es/product/mistertm
7. Siemens — Simcenter Computational Fluid Dynamics overview: https://www.siemens.com/en-us/products/simcenter/simulation-test/computational-fluid-dynamics/
8. ResearchGate — nozzle diameter / application-pattern reference supplied for research context: https://www.researchgate.net/figure/Nozzle-diameter-nozzle-sizes-of-the-db-water-applicatiom-pattern-widdthW-rs-and_tbl1_262664295

### Domain research context

9. Wageningen University & Research — greenhouse fogging and climate-control research. This is informative background only; the PoC target is **root-zone aeroponic nutrient delivery**, not greenhouse cooling/fogging.
10. Aeroponic literature on nozzle type, droplet size, misting interval, root-zone delivery, and plant response shall be treated as experimental evidence for calibration/validation rather than as a universal physical law.

## 1.5 Document Overview

Section 2 defines the product context, users, assumptions, and allocation. Section 3 defines external interfaces, functional requirements, quality requirements, compliance, implementation constraints, and AI/ML boundaries. Section 4 defines verification and the PoC acceptance gates. Section 5 contains supporting material and explicit non-goals.

# 2. Product Overview

## 2.1 Product Perspective

The PoC is the first domain-specific vertical slice of the CAD Integration Hub. The existing repository is intended to evolve from a 3D polygon-to-CAD bridge into a broader semantic bridge capable of representing physical systems that can be converted to CAD and simulation representations.

The target workflow is:

```text
Blender
  |
  | 3D geometry + semantic hints
  v
Semantic Extraction
  |
  v
CIM / Parametric Model
  |
  +--------------------+
  |                    |
  v                    v
CAD representation   Hydraulic model
                       |
                       v
                    CFD case
                       |
                       v
                  Droplet transport
                       |
                       v
                  Root-zone metrics
                       |
                       v
                  Parameter update
                       |
                       +----> CIM / CAD
```

The PoC must keep the CIM independent of Blender and independent of a particular CFD solver so that later adapters can target OpenCADStudio, Autodesk Inventor, SolidWorks, Creo, OpenFOAM, Siemens Simcenter, or other compatible systems.

## 2.2 Product Functions

1. Import or recognize a Blender microgreenhouse scene.
2. Identify semantic components: chamber, root zone, misters, nozzle instances, pipes, reservoir, pump, regulator, crop volumes, and relevant boundaries.
3. Construct a versioned CIM representation.
4. Convert Blender geometry into parametric engineering entities without requiring manual remodeling of the whole scene.
5. Associate real mister/nozzle metadata with model instances.
6. Represent pressure, flow, orientation, spray distribution, droplet population, and duty cycle as explicit parameters.
7. Build a hydraulic network representation for pressure/flow analysis.
8. Build a CFD fluid domain and droplet-injection definition.
9. Run a supported CFD workflow for airflow and Lagrangian/discrete droplet transport where the selected solver supports it.
10. Calculate root-zone delivery metrics and expose them to the user.
11. Modify parameters and regenerate simulation inputs without reconstructing the model from scratch.
12. Preserve provenance between source geometry, CIM entities, simulation inputs, and results.

## 2.3 Product Constraints

- The first PoC shall focus on **substrate-free aeroponic nutrient delivery to suspended roots**.
- Greenhouse fogging for climate cooling/humidity control is explicitly outside the primary objective.
- A mister shall not be modeled as if pressure alone uniquely determines droplet size.
- Initial CFD shall use an injection/distribution model for droplets rather than detailed internal nozzle atomization.
- Manufacturer and experimental data shall remain distinguishable from solver-derived data.
- The PoC shall be runnable without modifying the core of OpenCADStudio.
- The CIM shall be solver-neutral and CAD-target-neutral.
- Geometry and physical units shall be explicit and internally consistent.
- Numerical values used for demonstration that are not measured shall be clearly marked as illustrative assumptions.

## 2.4 User Characteristics

### CAD / mechanical designer
Needs to construct and adjust the physical system visually and parametrically, with minimal CFD expertise.

### Aeroponic system designer
Needs to place misters, define root zones, inspect hydraulic distribution, and evaluate nutrient delivery.

### CFD engineer
Needs explicit domains, boundary conditions, material properties, injection definitions, mesh controls, solver settings, and reproducible case inputs.

### Software engineer
Needs stable CIM schemas, APIs, adapters, validation, and automated tests.

### Researcher / agronomist
Needs traceable assumptions, experimental calibration data, and result metrics that can be compared with physical observations.

## 2.5 Assumptions and Dependencies

### Assumptions

- A representative Blender scene can be constructed with sufficiently clean object naming or metadata for semantic extraction.
- Mister manufacturer data or measured data can be supplied for the selected nozzle.
- Root geometry can initially be represented as a simplified porous/solid proxy rather than a complete botanical reconstruction.
- A CFD solver capable of the required continuous-phase and particle/droplet transport can be installed in the development environment.
- Initial validation can use synthetic/reference cases before physical experiments are available.

### Dependencies

- Blender and its Python API/add-on environment for scene-side extraction.
- Rust toolchain and workspace infrastructure.
- A selected CFD solver adapter.
- Experimental/manufacturer data for droplet distribution and hydraulic performance.
- Optional future OpenCADStudio integration through its external/plugin interfaces.

## 2.6 Apportioning of Requirements

| Capability | PoC increment |
|------------|--------------|
| Blender extraction | P0 |
| CIM schema | P0 |
| Mister parametric entity | P0 |
| Root-zone entity | P0 |
| Hydraulic network representation | P0 |
| CFD domain generation | P0 |
| Droplet injection | P0 |
| Root-zone metrics | P0 |
| Parameter sweep | P1 |
| Optimization | P1 |
| OpenCADStudio adapter | P1 |
| Experimental calibration loop | P1 |
| Inventor/SolidWorks/Creo adapters | Future |
| Detailed nozzle atomization | Future |

# 3. Requirements

## 3.1 External Interfaces

### 3.1.1 User Interfaces

The PoC UI shall provide, either through a desktop UI or a CLI initially, the ability to:

- load/select a Blender-derived model;
- inspect semantic entities;
- inspect and edit key parameters;
- select a mister/nozzle definition;
- create or regenerate a simulation case;
- run a simulation;
- inspect root-zone delivery metrics;
- compare parameter configurations;
- identify assumptions and data provenance.

A desktop UI (Bevy Engine + Iced RS) provides 3D visualization, parameter panels, CFD result overlays, and optimization controls.

### 3.1.2 Hardware Interfaces

No direct hardware control is required for P0. Future releases may interface with pumps, pressure sensors, flow meters, valves, environmental sensors, and data loggers for calibration and closed-loop control.

### 3.1.3 Software Interfaces

The system shall expose or consume the following logical interfaces:

- Blender scene extraction interface.
- Versioned CIM schema/API.
- Hydraulic solver interface.
- CFD solver adapter interface.
- Simulation result parser interface.
- Optional OpenCADStudio adapter interface.
- Optional MCP interface for agent-driven model/simulation operations.

## 3.2 Functional Requirements

### 3.2.1 Semantic extraction

- ID: REQ-FUNC-001
- Title: Extract aeroponic components
- Statement: The system shall identify supported aeroponic semantic components from a Blender scene.
- Rationale: The PoC must bridge visual modeling and engineering semantics.
- Acceptance Criteria: A canonical scene containing a chamber, root zone, two or more misters, and a fluid-supply network produces corresponding CIM entities with stable IDs.
- Verification Method: Demonstration + Test

### 3.2.2 Parametric CIM

- ID: REQ-FUNC-002
- Title: Represent parametric mister
- Statement: The system shall represent each mister with position, orientation, pressure, flow rate, nozzle identity, spray-pattern definition, droplet-population definition, and duty-cycle parameters.
- Rationale: These variables drive hydraulic and CFD analysis.
- Acceptance Criteria: Changing a mister pressure or position in the CIM changes generated simulation input without reconstructing the entity.
- Verification Method: Test

### 3.2.3 Root zone

- ID: REQ-FUNC-003
- Title: Represent suspended root zone
- Statement: The system shall represent the target root zone as an explicit simulation entity with geometry and an association to crop/root objects.
- Rationale: Nutrient delivery is evaluated at the roots, not at a generic greenhouse volume.
- Acceptance Criteria: CFD post-processing can classify droplet interactions or deposited mass against the root-zone region.
- Verification Method: Test

### 3.2.4 Hydraulic network

- ID: REQ-FUNC-004
- Title: Model fluid supply network
- Statement: The system shall represent pump, reservoir, regulator, manifold, pipes, branches, and mister connections when those components exist in the source model.
- Rationale: Mister pressure and flow are consequences of the hydraulic network, not isolated constants.
- Acceptance Criteria: The model can calculate or import pressure/flow values at individual mister nodes and preserve their provenance.
- Verification Method: Test / Analysis

### 3.2.5 CFD case generation

- ID: REQ-FUNC-005
- Title: Generate droplet-transport CFD case
- Statement: The system shall generate a solver-specific case from the CIM containing fluid-domain geometry, boundary conditions, material properties, mister injections, and droplet-population definitions.
- Rationale: The PoC must demonstrate CAD-to-simulation translation.
- Acceptance Criteria: A canonical model produces a reproducible solver input directory that can be executed by the selected CFD adapter.
- Verification Method: Test / Demonstration

### 3.2.6 Droplet population

- ID: REQ-FUNC-006
- Title: Preserve droplet distribution provenance
- Statement: The system shall represent droplet size distributions independently from pressure and shall identify whether each distribution is manufacturer-provided, experimentally measured, calibrated, assumed, or solver-derived.
- Rationale: Pressure does not universally determine droplet size.
- Acceptance Criteria: The simulation input records the source and parameters of the distribution used for every mister.
- Verification Method: Inspection + Test

### 3.2.7 Root-zone delivery metrics

- ID: REQ-FUNC-007
- Title: Calculate root-zone delivery
- Statement: The system shall calculate metrics for droplet delivery to the root zone from simulation results.
- Rationale: The PoC outcome must be biologically and engineering relevant.
- Acceptance Criteria: Results include at least root-zone delivery/deposition, spatial uniformity, evaporation/loss where supported, and wall/non-target deposition where supported.
- Verification Method: Test / Analysis

### 3.2.8 Parametric rerun

- ID: REQ-FUNC-008
- Title: Regenerate after parameter change
- Statement: The system shall allow supported parameters to be changed and shall regenerate affected simulation inputs without requiring a fresh semantic extraction.
- Rationale: This demonstrates the parametric CAD-to-CFD loop.
- Acceptance Criteria: At least pressure, mister position/orientation, and mister count can be changed and compared across two or more runs.
- Verification Method: Demonstration + Test

### 3.2.9 Parameter sweep

- ID: REQ-FUNC-009
- Title: Sweep mister parameters
- Statement: The system should support automated sweeps across selected pressure, placement, orientation, droplet-distribution, or duty-cycle variables.
- Rationale: Optimization requires systematic exploration of design space.
- Acceptance Criteria: A sweep produces a machine-readable table mapping input configurations to result metrics.
- Verification Method: Test

### 3.2.10 Optimization

- ID: REQ-FUNC-010
- Title: Rank candidate configurations
- Statement: The system should rank candidate configurations using a configurable objective combining root-zone delivery, uniformity, loss, and hydraulic/energy penalties.
- Rationale: The intended long-term value is design optimization, not merely visualization.
- Acceptance Criteria: Given at least three configurations, the system returns a deterministic ranking for a fixed objective and dataset.
- Verification Method: Test / Analysis

### 3.2.11 OpenCADStudio handoff

- ID: REQ-FUNC-011
- Title: Preserve CAD semantics for future adapters
- Statement: The CIM shall preserve enough geometry and semantic metadata to permit a future OpenCADStudio adapter without requiring re-extraction from Blender.
- Rationale: CAD Bridge is intended to be independent of any single CAD application.
- Acceptance Criteria: The canonical model can be serialized, reloaded, and passed to an adapter boundary with stable entity IDs and units.
- Verification Method: Test

## 3.3 Quality of Service

### 3.3.1 Performance

- ID: REQ-PERF-001
- Title: Deterministic model regeneration
- Statement: For a fixed source scene and parameter set, CIM serialization and simulation-input generation shall be deterministic apart from explicitly nondeterministic solver artifacts.
- Acceptance Criteria: Repeated generation produces equivalent normalized CIM and case manifests.
- Verification Method: Test

- ID: REQ-PERF-002
- Title: Incremental parameter updates
- Statement: A change to a parameter shall regenerate only the affected model/simulation artifacts where dependency information permits.
- Acceptance Criteria: The case manifest identifies changed inputs and does not require re-extraction of unchanged semantic entities.
- Verification Method: Test / Inspection

### 3.3.2 Security and integrity

- ID: REQ-SEC-001
- Title: No implicit external execution
- Statement: The bridge shall not execute external CFD or system commands without an explicit user or configured automation action.
- Acceptance Criteria: Solver execution requires an explicit run operation and records the invoked adapter/configuration.
- Verification Method: Test

- ID: REQ-SEC-002
- Title: Preserve input integrity
- Statement: Source geometry and CIM inputs shall not be silently overwritten by simulation results.
- Acceptance Criteria: Results are stored as separate artifacts linked to the source model revision.
- Verification Method: Test

### 3.3.3 Reliability

- ID: REQ-REL-001
- Title: Failed solver runs are recoverable
- Statement: A failed CFD run shall preserve the generated case, logs, input manifest, and failure status for diagnosis and rerun.
- Acceptance Criteria: A failed run can be inspected and rerun without reconstructing the source scene.
- Verification Method: Demonstration

### 3.3.4 Observability

- ID: REQ-OBS-001
- Title: Simulation provenance
- Statement: Every simulation result shall reference model revision, solver adapter, solver version where available, parameter manifest, and input-data provenance.
- Acceptance Criteria: A result artifact can be traced back to the CIM revision and mister data used to generate it.
- Verification Method: Inspection + Test

## 3.4 Compliance

- ID: REQ-COMP-001
- Title: License compatibility
- Statement: Dependencies and distributed artifacts shall be reviewed for license compatibility before inclusion in released PoC artifacts.
- Acceptance Criteria: A dependency/license inventory exists in repository documentation or CI output.
- Verification Method: Inspection

- ID: REQ-COMP-002
- Title: Data provenance
- Statement: Manufacturer data, experimental measurements, and literature-derived assumptions shall be clearly labeled and shall not be presented as equivalent evidence.
- Acceptance Criteria: Each external physical-data record contains source, date, units, and provenance category.
- Verification Method: Inspection

## 3.5 Design and Implementation

### 3.5.1 Installation

- ID: REQ-INST-001
- Title: Reproducible development environment
- Statement: The PoC shall document the Rust toolchain, Blender version/API assumptions, CFD solver prerequisites, and required environment configuration.
- Acceptance Criteria: A new developer can follow repository instructions to prepare the development environment and run the canonical non-CFD tests.
- Verification Method: Demonstration

### 3.5.2 Build and Delivery

- ID: REQ-BUILD-001
- Title: CI validation
- Statement: The repository shall provide automated checks for formatting, compilation, schema validation, unit tests, and deterministic fixture generation where applicable.
- Acceptance Criteria: CI passes on the canonical branch for the P0 software components.
- Verification Method: Test

### 3.5.3 Distribution

- ID: REQ-DIST-001
- Title: Local-first execution
- Statement: P0 shall support local execution without requiring a hosted backend or cloud CFD service.
- Acceptance Criteria: The canonical example can be prepared and simulated on a supported developer workstation with local dependencies.
- Verification Method: Demonstration

### 3.5.4 Maintainability

- ID: REQ-MAINT-001
- Title: Solver-neutral CIM
- Statement: Domain semantics shall not depend on a single CFD solver's native file format.
- Acceptance Criteria: CFD-specific serialization is confined to an adapter boundary.
- Verification Method: Inspection

### 3.5.5 Reusability

- ID: REQ-REUSE-001
- Title: Reusable physical entities
- Statement: Mister, root-zone, hydraulic-network, fluid-material, and droplet-population entities shall be reusable across multiple studies.
- Acceptance Criteria: A single mister definition can be referenced by multiple model instances or studies without duplication of its physical definition.
- Verification Method: Test

### 3.5.6 Portability

- ID: REQ-PORT-001
- Title: Cross-platform core
- Statement: The core CIM and simulation orchestration components should remain portable across supported desktop operating systems.
- Acceptance Criteria: Core Rust crates build on at least the primary development platform and one additional supported platform before P1.
- Verification Method: Test

### 3.5.7 Proof-of-concept constraints

- ID: REQ-POC-001
- Title: Vertical slice before breadth
- Statement: P0 shall prioritize one complete aeroponic workflow over broad support for multiple CAD or CFD platforms.
- Acceptance Criteria: The canonical Blender-to-CFD path works end-to-end before additional CAD target adapters are treated as complete.
- Verification Method: Demonstration

- ID: REQ-POC-002
- Title: No false physical precision
- Statement: The PoC shall distinguish simulated transport results from experimentally validated atomization characteristics.
- Acceptance Criteria: Documentation and result manifests identify which quantities are measured, manufacturer-supplied, assumed, or simulated.
- Verification Method: Inspection

### 3.5.8 Change management

- ID: REQ-CM-001
- Title: Versioned CIM
- Statement: CIM schema changes shall be versioned and backward-compatibility expectations shall be documented.
- Acceptance Criteria: A schema version is stored in every serialized CIM artifact.
- Verification Method: Test

## 3.6 AI/ML

AI/ML is not required to satisfy the P0 simulation loop.

- ID: REQ-ML-001
- Title: Optional agent control
- Statement: Future MCP/LLM interfaces may propose or execute parameter changes, but they shall operate through the same typed CIM and simulation APIs used by deterministic clients.
- Rationale: Agents must not bypass physical-model validation or provenance requirements.
- Acceptance Criteria: An agent-generated parameter proposal can be validated before execution and is recorded with provenance.
- Verification Method: Demonstration

- ID: REQ-ML-002
- Title: Human review for physical changes
- Statement: Any future automated optimization that could translate simulation results into real hardware settings shall provide an explicit review boundary before deployment to physical equipment.
- Acceptance Criteria: No P0 component directly controls a pump, valve, or pressure regulator from an optimization result.
- Verification Method: Inspection / Demonstration

# 4. Verification

## 4.1 Verification methods

| Method | Use |
|--------|-----|
| Test | Automated or scripted execution against expected outputs |
| Analysis | Numerical or logical evaluation of generated/simulated results |
| Inspection | Review of schemas, manifests, provenance, and artifacts |
| Demonstration | Human-observable end-to-end workflow |

## 4.2 Canonical PoC acceptance test

A canonical model shall contain:

- a microgreenhouse/aeroponic chamber;
- a suspended root zone;
- at least two mister instances;
- a reservoir;
- a pump or pressure-source representation;
- a simple pipe/manifold network;
- explicit fluid-domain boundaries;
- a droplet-population definition.

The acceptance workflow is:

1. Open the canonical Blender scene.
2. Extract semantic entities.
3. Serialize the CIM.
4. Validate geometry, units, topology, and required parameters.
5. Generate the hydraulic model.
6. Generate the CFD model.
7. Execute the supported solver workflow.
8. Parse results.
9. Calculate root-zone delivery and distribution metrics.
10. Change at least one mister parameter.
11. Regenerate the affected inputs.
12. Run the second case.
13. Compare both configurations.
14. Preserve both result manifests and provenance.

## 4.3 Verification matrix

| Requirement | Primary verification artifact |
|-------------|-------------------------------|
| REQ-FUNC-001 | Blender fixture + extraction test |
| REQ-FUNC-002 | CIM schema + parameter mutation test |
| REQ-FUNC-003 | Root-zone fixture + result classification test |
| REQ-FUNC-004 | Hydraulic fixture + network validation |
| REQ-FUNC-005 | CFD case fixture |
| REQ-FUNC-006 | Distribution provenance fixture |
| REQ-FUNC-007 | CFD result parser + metric test |
| REQ-FUNC-008 | Parameter rerun integration test |
| REQ-FUNC-009 | Sweep output fixture |
| REQ-FUNC-010 | Optimization/ranking test |
| REQ-FUNC-011 | CIM round-trip test |
| REQ-PERF-001 | Deterministic fixture comparison |
| REQ-REL-001 | Failed-run recovery test |
| REQ-OBS-001 | Result manifest inspection |
| REQ-COMP-001 | Dependency/license report |
| REQ-POC-001 | End-to-end demonstration |
| REQ-POC-002 | Physical-data provenance inspection |

## 4.4 Physical validation plan

P0 is a software/simulation proof of concept and shall not claim biological validation. P1 should introduce a physical test rig containing at minimum a selected mister/nozzle, pressure measurement, flow measurement, and a method for measuring or characterizing droplet distribution and root-zone deposition.

The physical validation loop should compare:

```text
Manufacturer / measured data
          |
          v
      CIM inputs
          |
          v
         CFD
          |
          v
   predicted delivery
          |
          v
   physical experiment
          |
          v
     calibration data
          |
          +----> updated model
```

The experiment shall be used to calibrate the injection/distribution representation before attempting to infer or optimize real-world operating settings.

# 5. Appendixes

## Appendix A — Canonical CIM entities

The initial semantic vocabulary should include:

```text
AeroponicSystem
├── Chamber
├── RootZone
├── Crop
├── Reservoir
├── Pump
├── PressureRegulator
├── Pipe
├── Manifold
├── Mister
├── NozzleDefinition
├── DropletPopulation
├── FluidMaterial
├── CFDStudy
├── BoundaryCondition
├── SimulationRun
└── CFDResult
```

### Mister

```text
Mister
├── id
├── position
├── orientation
├── pressure
├── flow_rate
├── nozzle_definition
├── spray_pattern
├── droplet_population
└── duty_cycle
```

### DropletPopulation

```text
DropletPopulation
├── distribution_model
├── d10
├── d50
├── d90
├── density
├── temperature
├── initial_velocity
└── provenance
```

## Appendix B — P0 physical model boundary

The initial model treats atomization as an input boundary/injection condition. It does not claim to resolve the internal breakup physics of a specific mister nozzle.

The model may simulate:

- continuous-phase airflow;
- gravity;
- turbulence where supported;
- droplet trajectories;
- droplet residence time;
- evaporation where supported;
- root-zone deposition;
- wall/non-target deposition;
- spatial delivery uniformity.

The model shall not silently claim to predict:

- exact droplet size from pressure alone;
- nozzle-specific breakup physics without calibration;
- nutrient uptake by roots from CFD alone;
- plant growth or yield from droplet transport alone.

## Appendix C — Candidate objective function

A future optimization objective may be represented as:

```text
score =
    w_delivery   * root_zone_delivery
  + w_uniformity * root_zone_uniformity
  - w_evap      * evaporation_loss
  - w_wall      * non_target_deposition
  - w_energy    * hydraulic_energy_cost
```

Weights and thresholds must be explicit study parameters. They shall not be hard-coded as biological truth.

## Appendix D — Example parameter set

The following is a schema example only and is **not** a recommended operating condition:

```yaml
system:
  chamber:
    length_m: 2.0
    width_m: 1.0
    height_m: 2.0

mister:
  pressure_bar: 3.0
  flow_l_h: null
  orientation_deg: [0, -90, 0]
  duty_cycle_s: 5

root_zone:
  representation: simplified_geometry

fluid:
  type: nutrient_solution
  density_kg_m3: null
  viscosity_pa_s: null
  surface_tension_n_m: null

notes:
  - Values must be replaced by selected hardware/manufacturer/experimental data before physical interpretation.
```

## Appendix E — Explicit non-goals

- Full-featured CAD replacement for Blender.
- Full-featured CFD GUI replacement for commercial solvers.
- Universal irrigation-product database.
- Universal pressure-to-droplet-size correlation.
- Direct autonomous control of physical aeroponic hardware in P0.
- Biological claims about plant health or yield without controlled experiments.
- Detailed nozzle internal atomization simulation in P0.

## Appendix F — Definition of Done for P0

P0 is complete when a clean checkout of the repository can demonstrate, using the canonical fixture:

1. Blender scene -> semantic CIM.
2. CIM -> validated parametric aeroponic model.
3. Parametric model -> hydraulic/CFD inputs.
4. CFD -> machine-readable results.
5. Results -> root-zone delivery metrics.
6. Parameter change -> regenerated case.
7. Second simulation -> comparable result set.
8. All artifacts retain model/version/data provenance.
9. Documentation clearly distinguishes assumptions from validated physical data.

The PoC is successful if it proves the **CAD → parametric CIM → simulation → results → parameter change → simulation** loop, even if detailed nozzle atomization and biological validation remain future work.
