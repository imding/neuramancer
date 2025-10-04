# Neuramancer App - Sequence Diagrams

This document contains sequence diagrams illustrating how the Neuramancer app acts as a neutral mediator in various interpersonal scenarios.

## Scenario 1: Romantic Partners Holding Back Resentments

```mermaid
sequenceDiagram
    participant Alex as Alex (Partner A)
    participant App as Neuramancer App
    participant Jordan as Jordan (Partner B)
    participant AI as AI Mediator

    Note over Alex, Jordan: Initial frustration building up
    
    Alex->>App: Records voice note: "Frustrated with last-minute cancellations"
    App->>App: Creates thought jar for Alex
    App->>AI: Analyzes emotional content and patterns
    
    Jordan->>App: Records text: "Sensing distance, feeling insecure"
    App->>App: Creates thought jar for Jordan
    App->>AI: Analyzes emotional content and patterns
    
    Note over Alex, Jordan: App detects relationship tension pattern
    
    AI->>App: Identifies communication breakdown pattern
    App->>Alex: Suggests creating link to Jordan's thoughts
    App->>Jordan: Suggests creating link to Alex's thoughts
    
    Alex->>App: Approves link with label "relationship_communication"
    Jordan->>App: Approves link with label "relationship_communication"
    
    App->>AI: Processes linked thoughts with "relationship_communication" context
    AI->>App: Generates neutral guidance
    
    App->>Alex: "Try 'I feel' statements to invite understanding"
    App->>Jordan: "Try 'I feel' statements to invite understanding"
    
    Note over Alex, Jordan: Both receive same guidance without exposing specific concerns
```

## Scenario 2: Close Friends Navigating Jealousy

```mermaid
sequenceDiagram
    participant Sam as Sam (Friend A)
    participant App as Neuramancer App
    participant Taylor as Taylor (Friend B)
    participant AI as AI Mediator

    Note over Sam, Taylor: Taylor gets promotion, tension builds
    
    Sam->>App: Records video: "Happy for Taylor but feeling left behind"
    App->>App: Creates thought jar for Sam
    App->>AI: Analyzes jealousy and self-doubt patterns
    
    Taylor->>App: Records audio: "Sam seems distant, worried about resentment"
    App->>App: Creates thought jar for Taylor
    App->>AI: Analyzes success guilt and friendship concerns
    
    AI->>App: Detects friendship strain pattern
    App->>Sam: Suggests linking with Taylor's thoughts
    App->>Taylor: Suggests linking with Sam's thoughts
    
    Sam->>App: Approves link with label "friendship_support"
    Taylor->>App: Approves link with label "friendship_support"
    
    App->>AI: Processes linked thoughts with "friendship_support" context
    AI->>App: Generates empathy-building guidance
    
    App->>Sam: "Celebrate wins by asking open questions about journeys"
    App->>Taylor: "Celebrate wins by asking open questions about journeys"
    
    Note over Sam, Taylor: Both learn to build empathy without comparison
```

## Scenario 3: Siblings Dealing with Family Role Expectations

```mermaid
sequenceDiagram
    participant Riley as Riley (Responsible Sibling)
    participant App as Neuramancer App
    participant Casey as Casey (Free Spirit Sibling)
    participant AI as AI Mediator

    Note over Riley, Casey: Family gathering logistics causing tension
    
    Riley->>App: Records text: "Always handling logistics, feeling taken for granted"
    App->>App: Creates thought jar for Riley
    App->>AI: Analyzes burden and resentment patterns
    
    Casey->>App: Records voice: "Feel guilty for leaning on Riley, avoiding events"
    App->>App: Creates thought jar for Casey
    App->>AI: Analyzes guilt and avoidance patterns
    
    AI->>App: Identifies family role imbalance pattern
    App->>Riley: Suggests creating link to Casey's perspective
    App->>Casey: Suggests creating link to Riley's perspective
    
    Riley->>App: Approves link with label "family_dynamics"
    Casey->>App: Approves link with label "family_dynamics"
    
    App->>AI: Processes linked thoughts with "family_dynamics" context
    AI->>App: Generates boundary-setting guidance
    
    App->>Riley: "Set gentle boundaries: 'I'd love help with this—want to team up?'"
    App->>Casey: "Set gentle boundaries: 'I'd love help with this—want to team up?'"
    
    Note over Riley, Casey: Both learn collaborative approaches to family responsibilities
```

## Scenario 4: Coworkers Grappling with Unspoken Credit Disputes

```mermaid
sequenceDiagram
    participant Mia as Mia (Idea Contributor)
    participant App as Neuramancer App
    participant Lee as Lee (Presenter)
    participant AI as AI Mediator

    Note over Mia, Lee: Project collaboration with credit issues
    
    Mia->>App: Records audio: "Ideas presented without acknowledgment, feeling invisible"
    App->>App: Creates thought jar for Mia
    App->>AI: Analyzes recognition and workplace dynamics
    
    Lee->>App: Records text: "Mia seems disengaged, confused by mixed signals"
    App->>App: Creates thought jar for Lee
    App->>AI: Analyzes collaboration confusion patterns
    
    AI->>App: Detects workplace credit/recognition pattern
    App->>Mia: Suggests linking with Lee's perspective
    App->>Lee: Suggests linking with Mia's perspective
    
    Mia->>App: Approves link with label "workplace_collaboration"
    Lee->>App: Approves link with label "workplace_collaboration"
    
    App->>AI: Processes linked thoughts with "workplace_collaboration" context
    AI->>App: Generates collaborative communication guidance
    
    App->>Mia: "Use 'we achieved this together' phrasing to foster shared ownership"
    App->>Lee: "Use 'we achieved this together' phrasing to foster shared ownership"
    
    Note over Mia, Lee: Both learn to create equitable credit sharing
```

## General App Flow Pattern

```mermaid
sequenceDiagram
    participant User1 as User 1
    participant App as Neuramancer App
    participant User2 as User 2
    participant AI as AI Mediator
    participant ThoughtJars as Thought Jars Storage

    User1->>App: Input thought (text/audio/video)
    App->>ThoughtJars: Store in User1's thought jar
    App->>AI: Analyze emotional patterns and content
    
    User2->>App: Input thought (text/audio/video)
    App->>ThoughtJars: Store in User2's thought jar
    App->>AI: Analyze emotional patterns and content
    
    AI->>App: Detect potential relationship pattern
    App->>User1: Suggest creating link to User2
    App->>User2: Suggest creating link to User1
    
    User1->>App: Approve link with descriptive label
    User2->>App: Approve link with descriptive label
    
    App->>AI: Process linked thoughts with label context
    AI->>App: Generate neutral, constructive guidance
    
    App->>User1: Deliver personalized but neutral advice
    App->>User2: Deliver personalized but neutral advice
    
    Note over User1, User2: Both receive guidance without exposing private thoughts
```

## Key Features Illustrated

1. **Privacy Preservation**: Individual thoughts remain private in separate "thought jars"
2. **Mutual Consent**: Both parties must approve links before information sharing
3. **Contextual Labels**: Labels like "relationship_communication" guide how the AI processes linked information
4. **Neutral Mediation**: AI provides guidance without revealing specific private thoughts
5. **Pattern Recognition**: AI identifies relationship dynamics and communication patterns
6. **Constructive Guidance**: Focus on building better communication skills rather than blame
