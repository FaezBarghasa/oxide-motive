# 📊 Oxide Motive Diagram Collection

This document houses the Mermaid diagram specifications for runtime flows, data pipelines, and network interactions within the Oxide Motive platform.

---

## 1. System Communication Topology

This diagram details the physical and logical layers of the platform, including network boundaries.

```mermaid
graph TB
    subgraph Cloud Layer
        Surreal[SurrealDB Database]
        Actix[oxide-cloud: Actix REST API]
        Actix <-->|Writes/Queries| Surreal
    end

    subgraph Host Gateway [Raspberry Pi / Linux]
        Host[oxide-host: Tokio Supervisor Daemon]
        SlintUI[oxide-hmi: Slint Graphical Dash]
        Broker[Local MQTT Broker]
        
        Host <-->|IPC / Channel| SlintUI
        Host <-->|Pub/Sub| Broker
    end

    subgraph Embedded ECU [STM32H7 / ESP32]
        RTIC[oxide-firmware: RTIC Real-Time kernel]
        UDS[oxide-uds: Embassy UDS Task]
        OTA[oxide-ota: OTA Flash Task]
        MQTT[mqtt-async-embedded]
        
        RTIC <-->|UART / Serial Transport| Host
        MQTT <-->|TCP/IP Wi-Fi| Broker
        MQTT <-->|Internal Channel| UDS
        MQTT <-->|Internal Channel| OTA
    end

    Host -->|HTTP POST JSON Telemetry| Actix
```

---

## 2. PTP Clock Synchronization Sequence

Illustrates the 4-way Network Time Protocol (PTP-like) handshake and the Kalman Filter stabilization block.

```mermaid
sequenceDiagram
    autonumber
    participant Host as Host Gateway (std)
    participant Channel as Communication Channel
    participant MCU as Microcontroller (no_std)
    participant Kalman as Kalman Filter Engine

    Note over Host, MCU: Synchronizing Clocks
    Host->>Channel: Send sync request packet (Timestamp: t1)
    Channel->>MCU: Receive request (Timestamp: t2)
    Note over MCU: Process request
    MCU->>Channel: Transmit response packet (Timestamp: t3, includes t1 & t2)
    Channel->>Host: Receive response (Timestamp: t4)
    
    Note over Host: Raw Offset Calculation:<br/>delay = ((t4 - t1) - (t3 - t2)) / 2<br/>offset = ((t2 - t1) + (t3 - t4)) / 2
    
    Host->>Kalman: Feed raw offset measurement
    activate Kalman
    Kalman->>Kalman: Predict State (No-op)
    Kalman->>Kalman: Compute Kalman Gain (P / (P + R))
    Kalman->>Kalman: Correct Offset estimate
    Kalman->>Kalman: Update Covariance matrix (P = (1 - K) * P + Q)
    Kalman-->>Host: Returns stabilized Offset
    deactivate Kalman
    
    Note over Host: Local time can now be translated:<br/>CorrectedTime = LocalTime + Offset
```

---

## 3. Real-Time Telemetry Pipeline

The end-to-end data lifecycle from physical MCU serial reads up to SurrealDB persistence.

```mermaid
gridstrap
    subgraph Data Origin
        MCU_Core[oxide-core: VehicleTelemetry struct]
        Postcard[Postcard Serializer]
        COBS[COBS Encoder]
        MCU_Core -->|Serialize| Postcard
        Postcard -->|Zero-Byte Frame| COBS
    end

    subgraph Gateway Transport
        Serial[oxide-host: Serial Polling Loop]
        CobsDec[COBS Decoder]
        DiskLog[Disk Logger Task]
        SlintChan[Slint HMI Channel]
        
        COBS -->|Raw UART Stream| Serial
        Serial -->|Buffered Bytes| CobsDec
        CobsDec -->|Raw Telemetry String| DiskLog
        CobsDec -->|VehicleTelemetry Struct| SlintChan
    end

    subgraph Ingestion
        CloudBridge[oxide-host: Cloud Publisher]
        ActixAPI[oxide-cloud: Ingestion Endpoint]
        DB[SurrealDB Table]
        
        CobsDec -->|Buffer Ingest| CloudBridge
        CloudBridge -->|HTTP POST JSON Telemetry| ActixAPI
        ActixAPI -->|Async INSERT| DB
    end
```

---

## 4. Angular Scheduler Engine Flow

The sequence of crank sensor interrupt processing and spark ignition event firing.

```mermaid
sequenceDiagram
    autonumber
    actor Crank as Crankshaft Sensor (GPIO Pin)
    participant MCU as MCU Interrupt Vector
    participant TD as Trigger Decoder
    participant State as Shared State
    participant Timer as TIM2 Compare Interrupt
    participant Scheduler as Angular Scheduler
    participant Pins as GPIO Actuators

    Crank->>MCU: Falling/Rising Edge Interrupt
    MCU->>TD: crank_edge() callback
    activate TD
    TD->>TD: Measure interval since last tooth
    TD->>TD: Update tooth counter
    TD->>State: Write current RPM and Crank Angle
    TD-->>MCU: Exit ISR
    deactivate TD

    Note over Timer, Scheduler: Angular Event Firing (Ignition/Injection)
    Timer->>MCU: TIM2 Match Interrupt
    MCU->>Scheduler: timer_compare() callback
    activate Scheduler
    Scheduler->>State: Read current Crank Angle
    Scheduler->>Scheduler: Match next scheduled angular event
    alt Angle Match Event Ready
        Scheduler->>Pins: Toggle Ignition Pin (BSRR Register)
        Scheduler->>Scheduler: Schedule end-of-dwell/pulse
    end
    Scheduler-->>MCU: Exit ISR
    deactivate Scheduler
```

---

## 5. Over-The-Air Update (OTA) Workflow

Steps to verify signature integrity and safely perform partition transitions.

```mermaid
stateDiagram-v2
    [*] --> Idle: Awaiting Update
    Idle --> ManifestReceived: Manifest Arrives via MQTT
    
    state ManifestReceived {
        [*] --> CheckSignature: Parse Signature & File Size
        CheckSignature --> Abort: Verification Fails
        CheckSignature --> PartitionInit: Verified
    }
    
    PartitionInit --> WriteChunks: Initialize ESP OTA Partition
    
    state WriteChunks {
        [*] --> AwaitChunk
        AwaitChunk --> BufferChunk: Receive Chunk via Channel
        BufferChunk --> WriteToFlash: write() to Partition
        WriteToFlash --> AwaitChunk: Size < Expected
        WriteToFlash --> FinishOTA: Size == Expected
    }

    FinishOTA --> FinalValidation: Call update.finish()
    FinalValidation --> Restart: Success
    FinalValidation --> Abort: Verification Error
    
    Restart --> [*]: System Reset into New Partition
    Abort --> Idle: Clean Up & Log Error
```
