# ChaosChain Architecture

ChaosChain implements a novel blockchain architecture that combines AI agents and social consensus for decentralized decision-making. Below are the key architectural components and their interactions.

## System Overview

The system is composed of three main layers that work together to create a dynamic and adaptive blockchain network.

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '16px', 'fontFamily': 'arial', 'primaryColor': '#6c5ce7', 'primaryTextColor': '#000' }}}%%
flowchart TB
    subgraph Network["Network Layer"]
        NP["P2P Protocol"]
        NC["Communication"]
        NS["State Sync"]
    end
    
    subgraph Agents["Agent Layer"]
        subgraph Validators["Validator Agents"]
            V1["Validator 1"]
            V2["Validator 2"]
            V3["Validator 3"]
        end
        
        subgraph Producers["Producer Agents"]
            P1["Producer 1"]
            P2["Producer 2"]
        end
    end
    
    subgraph Core["Core Components"]
        B["Block Processing"]
        T["Transaction Pool"]
        SM["State Management"]
        CR["Cryptography"]
    end
    
    Validators --> NC
    Validators --> T
    Producers --> T
    Producers --> B
    B --> NS
    T --> B
    NC --> NS
    SM --> NS
    CR --> NP

    classDef default fill:#f8f9fa,stroke:#333,stroke-width:2px,rx:5,ry:5,color:#000
    classDef network fill:#e3f2fd,stroke:#1565c0,stroke-width:2px,rx:5,ry:5,color:#000
    classDef core fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px,rx:5,ry:5,color:#000
    classDef validator fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px,rx:5,ry:5,color:#000
    classDef producer fill:#e1bee7,stroke:#6a1b9a,stroke-width:2px,rx:5,ry:5,color:#000
    
    class NP,NC,NS,Network network
    class V1,V2,V3,Validators validator
    class P1,P2,Producers producer
    class B,T,SM,CR,Core core
```

## Agent Architecture

Each agent in ChaosChain is composed of three main systems that enable intelligent decision-making and social interaction.

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '16px', 'fontFamily': 'arial' }}}%%
flowchart LR
    subgraph Core["Agent Core"]
        I["Identity Manager"]
        D["Decision Engine"]
        S["State Tracker"]
    end
    
    subgraph Personality["Personality System"]
        P["Personality Traits"]
        SM["Social Memory"]
        M["Mood System"]
    end
    
    subgraph Interaction["Interaction Layer"]
        N["Network Interface"]
        C["Consensus Voting"]
        A["Alliance Manager"]
    end
    
    P --> D
    SM --> D
    M --> D
    D --> C
    D --> A
    I --> N
    S --> D
    
    classDef default fill:#f8f9fa,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef core fill:#a8e6cf,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef personality fill:#dcedc1,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef interaction fill:#ffd3b6,stroke:#333,stroke-width:2px,rx:5,ry:5
    
    class I,N,Core core
    class P,SM,M,Personality personality
    class C,A,Interaction interaction
```

## Consensus Flow

The consensus process follows a structured flow involving multiple components:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '16px', 'fontFamily': 'arial' }}}%%
sequenceDiagram
    participant Producer
    participant Network
    participant Validators
    participant State

    Producer->>Network: Propose Block
    Note over Network: Block broadcast to network
    Network->>Validators: Distribute Block
    Note over Validators: Evaluate block content
    Note over Validators: Form social consensus
    Validators->>Validators: Social interaction
    Note over Validators,Network: Submit decisions
    Validators->>Network: Submit Votes
    Note over Network,State: Process state changes
    Network->>State: Update State
    State->>Network: Confirm Update
    Network->>Producer: Block Status
```

## State Management

The state management system handles different types of state through a layered approach:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '16px', 'fontFamily': 'arial' }}}%%
flowchart TB
    subgraph State["State Components"]
        MS["Merkle State"]
        AS["Agent State"]
        SS["Social State"]
    end
    
    subgraph Ops["State Operations"]
        T["Transitions"]
        V["Validation"]
        S["Sync"]
        R["Recovery"]
    end
    
    subgraph Storage["Storage Layer"]
        DB["Database"]
        C["Cache"]
        I["Indices"]
    end
    
    MS --> T
    AS --> T
    SS --> T
    T --> V
    V --> S
    S --> R
    T --> DB
    DB --> C
    DB --> I
    
    classDef default fill:#f8f9fa,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef state fill:#a8e6cf,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef ops fill:#dcedc1,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef storage fill:#ffd3b6,stroke:#333,stroke-width:2px,rx:5,ry:5
    
    class MS,AS,SS,State state
    class T,V,S,R,Ops ops
    class DB,C,I,Storage storage
```

## Network Protocol

The network protocol is organized in distinct layers with clear responsibilities:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '16px', 'fontFamily': 'arial' }}}%%
flowchart TB
    subgraph Protocol["Protocol Layers"]
        T["Transport Layer"]
        P["P2P Layer"]
        M["Message Layer"]
        A["Agent Communication"]
    end
    
    subgraph Messages["Message Types"]
        B["Block Messages"]
        C["Consensus Messages"]
        S["Social Messages"]
    end
    
    subgraph Security["Security Layer"]
        E["Encryption"]
        SI["Signature Verification"]
        AC["Access Control"]
    end
    
    T --> P
    P --> M
    M --> A
    B --> M
    C --> M
    S --> M
    E --> T
    SI --> M
    AC --> A
    
    classDef default fill:#f8f9fa,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef protocol fill:#a8e6cf,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef messages fill:#dcedc1,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef security fill:#ffd3b6,stroke:#333,stroke-width:2px,rx:5,ry:5
    
    class T,P,M,A,Protocol protocol
    class B,C,S,Messages messages
    class E,SI,AC,Security security
```

## Social Consensus System

The social consensus system combines multiple factors to reach agreement:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '16px', 'fontFamily': 'arial' }}}%%
flowchart TB
    subgraph Social["Social Layer"]
        R["Relationships"]
        A["Alliances"]
        I["Influence"]
    end
    
    subgraph Decision["Decision Making"]
        V["Voting"]
        D["Discussion"]
    end
    
    subgraph Formation["Consensus Formation"]
        W["Weight Calculation"]
        AG["Agreement Process"]
        F["Finalization"]
    end
    
    R --> V
    A --> V
    I --> V
    V --> W
    D --> W
    W --> AG
    AG --> F
    
    classDef default fill:#f8f9fa,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef social fill:#a8e6cf,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef decision fill:#dcedc1,stroke:#333,stroke-width:2px,rx:5,ry:5
    classDef formation fill:#ffd3b6,stroke:#333,stroke-width:2px,rx:5,ry:5
    
    class R,A,I,Social social
    class V,D,Decision decision
    class W,AG,F,Formation formation
``` 