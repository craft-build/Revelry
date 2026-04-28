# Architecture Patterns: Tauri Desktop Applications

**Project:** Persona - AI Roleplay Desktop Application  
**Stack:** Tauri 2.0 + React + TypeScript + Rust + SQLite  
**Researched:** 2025-04-27  
**Confidence:** HIGH

---

## Executive Summary

Tauri applications follow a **multi-process architecture** with three distinct layers:

1. **Frontend Process** (Webview) - React/TypeScript UI running in native webview
2. **Core Process** (Rust) - Application logic, security boundary, system access
3. **System APIs** - OS-level services (keychain, file system, network)

For a React + SQLite application like Persona, the recommended pattern is:
- **Frontend:** Single-page application (SPA) with React Query/TanStack Query for server-state
- **Backend:** Modular Rust commands with embedded SQLite via `tauri-plugin-sql`
- **IPC:** Commands for data operations, Events/Channels for streaming (AI responses)
- **State:** Zustand for client state, SQLite for persistence, Rust state for session data

---

## 1. Tauri IPC Patterns: Commands vs Events vs Channels

### Core IPC Primitives

Tauri uses **asynchronous message passing** (JSON-RPC under the hood) between frontend and backend:

```
┌─────────────────────────────────────────────────────────────┐
│                     IPC Architecture                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Frontend (Webview)          Core Process (Rust)           │
│   ┌─────────────────┐         ┌──────────────────────┐     │
│   │   React SPA     │         │  Commands & Events   │     │
│   │  ┌───────────┐  │         │  ┌────────────────┐  │     │
│   │  │  invoke   │──┼────────▶│  │ #[tauri::cmd]  │  │     │
│   │  └───────────┘  │         │  └────────────────┘  │     │
│   │       ▲         │         │          │           │     │
│   │  ┌────┴────┐    │         │          ▼           │     │
│   │  │ listen  │◀───┼─────────│  emit / channel      │     │
│   │  └─────────┘    │         │  ┌────────────────┐  │     │
│   │                 │         │  │ Channel<T>     │  │     │
│   └─────────────────┘         │  └────────────────┘  │     │
│                               └──────────────────────┘     │
│                                        │                    │
│                                        ▼                    │
│                              ┌──────────────────────┐      │
│                              │   SQLite / Keychain  │      │
│                              └──────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### Commands (Request/Response Pattern)

**Use for:** CRUD operations, one-off data fetching, authenticated actions

```rust
// Rust backend - src-tauri/src/commands/chat.rs
#[derive(serde::Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: String,
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub content: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn send_message(
    req: SendMessageRequest,
    db: tauri::State<'_, DatabaseConnection>,
) -> Result<MessageResponse, String> {
    // Validate, save to DB, return result
    let message = db.create_message(&req).await
        .map_err(|e| e.to_string())?;
    Ok(message.into())
}
```

```typescript
// Frontend - src/api/chat.ts
import { invoke } from '@tauri-apps/api/core';

interface SendMessageRequest {
  conversationId: string;
  content: string;
}

interface MessageResponse {
  id: string;
  content: string;
  createdAt: string;
}

export async function sendMessage(
  req: SendMessageRequest
): Promise<MessageResponse> {
  return invoke('send_message', {
    conversationId: req.conversationId,
    content: req.content,
  });
}
```

### Events (Pub/Sub Pattern)

**Use for:** Lifecycle notifications, state changes, simple broadcasts

```rust
// Backend emitting
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn notify_progress(app: AppHandle, progress: u32) {
    app.emit("download-progress", progress).unwrap();
}
```

```typescript
// Frontend listening
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<number>('download-progress', (event) => {
  console.log(`Progress: ${event.payload}%`);
});

// Cleanup on component unmount
unlisten();
```

### Channels (Streaming Pattern)

**Use for:** AI response streaming, file uploads/downloads, real-time updates

```rust
// Backend with channel for streaming AI responses
use tauri::ipc::Channel;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(tag = "type", content = "data")]
enum StreamEvent {
    Token(String),
    Complete,
    Error(String),
}

#[tauri::command]
async fn stream_chat_response(
    message: String,
    on_event: Channel<StreamEvent>,
) -> Result<(), String> {
    // Stream tokens from AI API
    for token in ai_stream(&message).await {
        on_event.send(StreamEvent::Token(token)).unwrap();
    }
    on_event.send(StreamEvent::Complete).unwrap();
    Ok(())
}
```

```typescript
// Frontend consuming stream
import { invoke, Channel } from '@tauri-apps/api/core';

type StreamEvent =
  | { type: 'token'; data: string }
  | { type: 'complete' }
  | { type: 'error'; data: string };

const channel = new Channel<StreamEvent>();
channel.onmessage = (event) => {
  switch (event.type) {
    case 'token':
      appendToMessage(event.data);
      break;
    case 'complete':
      finalizeMessage();
      break;
    case 'error':
      handleError(event.data);
      break;
  }
};

await invoke('stream_chat_response', { message: userInput, onEvent: channel });
```

### Decision Matrix

| Pattern | Latency | Payload Size | Direction | Use Case |
|---------|---------|--------------|-----------|----------|
| **Commands** | Medium | Large (JSON) | Frontend → Backend → Frontend | CRUD, auth, configuration |
| **Events** | Low | Small (JSON) | Bidirectional | Status updates, notifications |
| **Channels** | Very Low | Large (binary) | Backend → Frontend | Streaming AI, files, logs |

---

## 2. React Frontend Architecture for Desktop Apps

### Recommended Project Structure

```
frontend/
├── src/
│   ├── components/           # UI components
│   │   ├── ui/              # Primitive components (Radix-based)
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   └── dialog.tsx
│   │   ├── layout/          # Layout components
│   │   │   ├── sidebar.tsx
│   │   │   ├── header.tsx
│   │   │   └── main-content.tsx
│   │   └── chat/            # Feature components
│   │       ├── message-list.tsx
│   │       ├── message-input.tsx
│   │       └── streaming-message.tsx
│   ├── hooks/               # Custom React hooks
│   │   ├── use-conversation.ts
│   │   ├── use-characters.ts
│   │   ├── use-streaming-response.ts
│   │   └── use-theme.ts
│   ├── lib/                 # Utilities
│   │   ├── api.ts           # Tauri command wrappers
│   │   ├── query-client.ts  # React Query setup
│   │   └── utils.ts
│   ├── stores/              # State management (Zustand)
│   │   ├── ui-store.ts      # Sidebar state, modals
│   │   └── chat-store.ts    # Current conversation state
│   ├── types/               # TypeScript types
│   │   ├── character.ts
│   │   ├── conversation.ts
│   │   └── message.ts
│   ├── App.tsx
│   └── main.tsx
├── index.html
├── package.json
└── vite.config.ts
```

### State Management Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                    State Architecture                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌──────────────┐  ┌──────────────┐  ┌────────────────┐   │
│   │   Zustand    │  │ React Query  │  │ Local State    │   │
│   │  (UI State)  │  │ (Server State│  │  (Component)   │   │
│   └──────┬───────┘  └──────┬───────┘  └────────────────┘   │
│          │                 │                                │
│          ▼                 ▼                                │
│   ┌────────────────────────────────────┐                   │
│   │           Tauri Commands           │                   │
│   │   ┌────────────────────────────┐   │                   │
│   │   │  invoke('get_conversations')│   │                   │
│   │   │  invoke('send_message')    │   │                   │
│   │   └────────────────────────────┘   │                   │
│   └────────────────────────────────────┘                   │
│                        │                                    │
│                        ▼                                    │
│               ┌──────────────────┐                          │
│               │   SQLite + Rust  │                          │
│               └──────────────────┘                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Zustand** for client state (UI, ephemeral data):

```typescript
// stores/ui-store.ts
import { create } from 'zustand';

interface UIState {
  sidebarOpen: boolean;
  currentView: 'chat' | 'characters' | 'settings';
  activeConversationId: string | null;
  toggleSidebar: () => void;
  setView: (view: 'chat' | 'characters' | 'settings') => void;
  setActiveConversation: (id: string | null) => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: true,
  currentView: 'chat',
  activeConversationId: null,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setView: (view) => set({ currentView: view }),
  setActiveConversation: (id) => set({ activeConversationId: id }),
}));
```

**TanStack Query (React Query)** for server-state:

```typescript
// hooks/use-conversations.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function useConversations() {
  return useQuery({
    queryKey: ['conversations'],
    queryFn: () => invoke<Conversation[]>('get_conversations'),
  });
}

export function useCreateConversation() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateConversationInput) =>
      invoke<Conversation>('create_conversation', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['conversations'] });
    },
  });
}
```

### Streaming AI Response Pattern

```typescript
// hooks/use-streaming-response.ts
import { useState, useCallback } from 'react';
import { invoke, Channel } from '@tauri-apps/api/core';

interface UseStreamingResponseOptions {
  onToken?: (token: string) => void;
  onComplete?: () => void;
  onError?: (error: string) => void;
}

export function useStreamingResponse(options: UseStreamingResponseOptions = {}) {
  const [isStreaming, setIsStreaming] = useState(false);
  const [partialResponse, setPartialResponse] = useState('');

  const sendMessage = useCallback(async (content: string) => {
    setIsStreaming(true);
    setPartialResponse('');

    const channel = new Channel<StreamEvent>();
    channel.onmessage = (event) => {
      switch (event.type) {
        case 'token':
          setPartialResponse((prev) => prev + event.data);
          options.onToken?.(event.data);
          break;
        case 'complete':
          setIsStreaming(false);
          options.onComplete?.();
          break;
        case 'error':
          setIsStreaming(false);
          options.onError?.(event.data);
          break;
      }
    };

    await invoke('send_message_stream', { content, onEvent: channel });
  }, [options]);

  return { sendMessage, isStreaming, partialResponse };
}
```

---

## 3. Rust Backend Module Organization

### Recommended Project Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Entry point (desktop only)
│   ├── lib.rs               # Library entry, mobile + desktop
│   ├── commands/            # All Tauri commands
│   │   ├── mod.rs
│   │   ├── chat.rs          # Conversation & message commands
│   │   ├── character.rs     # Character CRUD
│   │   ├── provider.rs      # AI provider settings
│   │   ├── settings.rs      # App settings
│   │   └── search.rs        # FTS search commands
│   ├── models/              # Data structures
│   │   ├── mod.rs
│   │   ├── conversation.rs
│   │   ├── message.rs
│   │   ├── character.rs
│   │   └── provider.rs
│   ├── db/                  # Database layer
│   │   ├── mod.rs
│   │   ├── migrations.rs    # SQL migrations
│   │   ├── connection.rs    # DB pool/connection
│   │   ├── conversation.rs  # Conversation queries
│   │   ├── message.rs       # Message queries
│   │   └── character.rs     # Character queries
│   ├── services/            # Business logic
│   │   ├── mod.rs
│   │   ├── ai_client.rs     # OpenAI-compatible client
│   │   ├── streaming.rs     # Response streaming
│   │   ├── character_card.rs # PNG metadata parsing
│   │   └── encryption.rs    # Vault mode encryption
│   ├── state/               # Application state
│   │   ├── mod.rs
│   │   └── app_state.rs
│   └── error.rs             # Error types
├── migrations/              # SQL migration files
│   ├── 001_initial.sql
│   ├── 002_conversation_branching.sql
│   └── 003_lorebooks.sql
├── capabilities/            # Tauri capability configs
│   └── default.json
├── Cargo.toml
└── tauri.conf.json
```

### Command Module Pattern

```rust
// src/commands/chat.rs
use tauri::State;
use crate::db::Database;
use crate::models::{Message, CreateMessageRequest};
use crate::error::AppError;

#[tauri::command]
pub async fn get_conversations(
    db: State<'_, Database>,
) -> Result<Vec<ConversationSummary>, AppError> {
    db.get_conversations().await.map_err(Into::into)
}

#[tauri::command]
pub async fn get_messages(
    conversation_id: String,
    db: State<'_, Database>,
) -> Result<Vec<Message>, AppError> {
    db.get_messages(&conversation_id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn send_message(
    request: CreateMessageRequest,
    db: State<'_, Database>,
    ai: State<'_, AiClient>,
) -> Result<Message, AppError> {
    // Save user message
    let user_msg = db.create_message(&request).await?;
    
    // Get AI response
    let ai_response = ai.generate_response(&request.content).await?;
    
    // Save AI message
    let ai_msg = db.create_message(&CreateMessageRequest {
        conversation_id: request.conversation_id,
        content: ai_response,
        role: "assistant".into(),
    }).await?;
    
    Ok(ai_msg)
}

#[tauri::command]
pub async fn send_message_stream(
    request: CreateMessageRequest,
    on_event: tauri::ipc::Channel<StreamEvent>,
    db: State<'_, Database>,
    ai: State<'_, AiClient>,
) -> Result<(), AppError> {
    // Implementation using channels for streaming
    // ...
}
```

```rust
// src/lib.rs - Command registration
mod commands;
mod db;
mod models;
mod services;
mod state;
mod error;

use commands::{chat, character, provider, settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Initialize database
            let db = db::init(app)?;
            app.manage(db);
            
            // Initialize AI client
            let ai_client = services::AiClient::new(app)?;
            app.manage(ai_client);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            chat::get_conversations,
            chat::get_messages,
            chat::send_message,
            chat::send_message_stream,
            chat::create_conversation,
            chat::delete_conversation,
            // Character commands
            character::get_characters,
            character::create_character,
            character::import_character,
            character::export_character,
            // Provider commands
            provider::get_providers,
            provider::set_api_key,
            provider::test_connection,
            // Settings commands
            settings::get_settings,
            settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Error Handling Pattern

```rust
// src/error.rs
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("AI client error: {0}")]
    AiClient(String),
    
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("not found: {0}")]
    NotFound(String),
}

// Implement Serialize for AppError to return structured errors to frontend
#[derive(Serialize)]
#[serde(tag = "type", content = "message")]
pub enum ErrorResponse {
    Database(String),
    AiClient(String),
    Validation(String),
    NotFound(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let response = match self {
            AppError::Database(msg) => ErrorResponse::Database(msg.to_string()),
            AppError::AiClient(msg) => ErrorResponse::AiClient(msg.clone()),
            AppError::Validation(msg) => ErrorResponse::Validation(msg.clone()),
            AppError::NotFound(msg) => ErrorResponse::NotFound(msg.clone()),
        };
        response.serialize(serializer)
    }
}
```

---

## 4. SQLite Schema Design for Chat Applications

### Database Strategy

**Use `tauri-plugin-sql`** with sqlx for type-safe queries:

```toml
# Cargo.toml
[dependencies]
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
```

### Recommended Schema

```sql
-- migrations/001_initial.sql

-- Characters (AI personas)
CREATE TABLE characters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    personality TEXT,
    scenario TEXT,
    first_mes TEXT,
    mes_example TEXT,
    creatorcomment TEXT,
    tags TEXT, -- JSON array
    creator TEXT,
    character_version TEXT,
    extensions TEXT, -- JSON object for CharacterCardV2 extras
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Conversations (chat sessions)
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    character_id TEXT REFERENCES characters(id) ON DELETE SET NULL,
    title TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    -- For conversation branching
    parent_conversation_id TEXT REFERENCES conversations(id) ON DELETE CASCADE,
    -- Metadata
    is_pinned BOOLEAN DEFAULT FALSE,
    metadata TEXT -- JSON for extensibility
);

-- Messages (conversation content)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    -- For branching support
    branch_id INTEGER DEFAULT 0, -- 0 = main branch
    -- Metadata
    token_count INTEGER,
    model TEXT, -- Which model generated this
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    -- Soft delete for undo
    is_deleted BOOLEAN DEFAULT FALSE
);

-- API Providers (OpenAI-compatible endpoints)
CREATE TABLE providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key_reference TEXT, -- Reference to OS keychain entry
    is_enabled BOOLEAN DEFAULT TRUE,
    is_default BOOLEAN DEFAULT FALSE,
    -- Configuration
    default_model TEXT,
    max_tokens INTEGER,
    temperature REAL DEFAULT 0.7,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Lorebooks (character knowledge)
CREATE TABLE lorebooks (
    id TEXT PRIMARY KEY,
    character_id TEXT REFERENCES characters(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_recursive BOOLEAN DEFAULT FALSE, -- Scan generated content too
    scan_depth INTEGER DEFAULT 50, -- How many messages to scan
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Lorebook entries
CREATE TABLE lorebook_entries (
    id TEXT PRIMARY KEY,
    lorebook_id TEXT NOT NULL REFERENCES lorebooks(id) ON DELETE CASCADE,
    keys TEXT NOT NULL, -- JSON array of keywords to match
    content TEXT NOT NULL,
    order_index INTEGER DEFAULT 0,
    enabled BOOLEAN DEFAULT TRUE,
    case_sensitive BOOLEAN DEFAULT FALSE,
    -- Position in prompt
    insert_position TEXT DEFAULT 'after_char' CHECK (insert_position IN (
        'after_char', 'before_char', 'after_example', 'before_example'
    ))
);

-- Full-text search virtual tables
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content='messages',
    content_rowid='rowid'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;

CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) 
    VALUES ('delete', old.rowid, old.content);
END;

CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) 
    VALUES ('delete', old.rowid, old.content);
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for common queries
CREATE INDEX idx_messages_conversation ON messages(conversation_id);
CREATE INDEX idx_messages_parent ON messages(parent_message_id);
CREATE INDEX idx_messages_created ON messages(created_at);
CREATE INDEX idx_conversations_character ON conversations(character_id);
CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);
```

### Migration Management

```rust
// src/db/migrations.rs
use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../../migrations/001_initial.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_conversation_branching",
            sql: include_str!("../../migrations/002_conversation_branching.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "add_lorebooks",
            sql: include_str!("../../migrations/003_lorebooks.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
```

```rust
// src/db/mod.rs
use tauri::AppHandle;
use tauri_plugin_sql::{Builder as SqlBuilder, Migration};

pub struct Database {
    connection: tauri_plugin_sql::Database,
}

pub fn init(app: &AppHandle) -> Result<Database, Box<dyn std::error::Error>> {
    let migrations = migrations();
    
    let connection = SqlBuilder::default()
        .add_migrations("sqlite:persona.db", migrations)
        .build();
    
    // Preload connection on startup
    tauri::async_runtime::block_on(async {
        connection.load("sqlite:persona.db").await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    
    Ok(Database { connection })
}
```

---

## 5. State Management Across Frontend/Backend Boundary

### Three-Layer State Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                        State Layers                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │   Frontend       │  │    Rust Core     │  │   Persistence    │  │
│  │   (React/Zustand)│  │    (Tauri State) │  │   (SQLite/Store) │  │
│  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘  │
│           │                     │                     │             │
│           │  UI State           │  Session State      │  Persistent │
│           │  - Sidebar open     │  - DB connection    │  - Messages │
│           │  - Active modal     │  - AI client        │  - Settings │
│           │  - Draft inputs     │  - HTTP clients     │  - Characters│
│           │                     │                     │             │
│           │                     │                     │             │
│           └─────────────────────┴─────────────────────┘             │
│                          Sync via IPC                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Rust State Management

```rust
// src/state/app_state.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    // Database connection pool
    pub db: Arc<RwLock<Database>>,
    
    // AI client configuration
    pub ai_client: Arc<RwLock<AiClient>>,
    
    // In-memory cache for frequently accessed data
    pub cache: Arc<RwLock<Cache>>,
    
    // Current streaming sessions (for cancellation)
    pub active_streams: Arc<RwLock<HashMap<String, AbortHandle>>>,
}

impl AppState {
    pub fn new(db: Database, ai_client: AiClient) -> Self {
        Self {
            db: Arc::new(RwLock::new(db)),
            ai_client: Arc::new(RwLock::new(ai_client)),
            cache: Arc::new(RwLock::new(Cache::new())),
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
```

```rust
// Register state in lib.rs
.setup(|app| {
    let db = db::init(app)?;
    let ai_client = services::AiClient::new(app)?;
    let state = AppState::new(db, ai_client);
    app.manage(state);
    Ok(())
})
```

### Accessing State in Commands

```rust
use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn get_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>, AppError> {
    // Read lock for concurrent access
    let db = state.db.read().await;
    db.get_conversations().await
}

#[tauri::command]
pub async fn update_ai_config(
    config: AiConfig,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Write lock for mutation
    let mut ai_client = state.ai_client.write().await;
    ai_client.update_config(config).await
}
```

### Frontend State Sync Pattern

```typescript
// lib/api.ts - Type-safe API layer
import { invoke } from '@tauri-apps/api/core';
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      refetchOnWindowFocus: false,
    },
  },
});

// Generic invoke wrapper with error handling
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    // Transform Tauri errors into application errors
    throw new AppError(error);
  }
}
```

---

## 6. File System Access Patterns in Tauri

### Security-First Approach

Tauri's file system access is **deny-by-default** with explicit allowlists:

```json
// capabilities/default.json
{
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions": [
    "fs:default",
    "fs:allow-app-read-recursive",
    "fs:allow-app-write-recursive",
    "fs:allow-download-read",
    "fs:allow-download-write"
  ]
}
```

### Base Directories Strategy

```typescript
import {
  BaseDirectory,
  readTextFile,
  writeTextFile,
  readDir,
  mkdir,
} from '@tauri-apps/plugin-fs';

// Recommended base directories for Persona:
const APP_DIRS = {
  // Database, config files
  CONFIG: BaseDirectory.AppConfig,
  // Character images, exports
  DATA: BaseDirectory.AppData,
  // Cached responses
  CACHE: BaseDirectory.AppCache,
  // Application logs
  LOG: BaseDirectory.AppLog,
  // User exports (backups)
  DOWNLOAD: BaseDirectory.Download,
};

// Reading character card PNGs
export async function readCharacterCard(path: string): Promise<Character> {
  const contents = await readFile(path);
  // Parse PNG metadata...
  return parseCharacterCard(contents);
}

// Exporting conversations
export async function exportConversation(
  conversation: Conversation,
  filename: string
): Promise<void> {
  const json = JSON.stringify(conversation, null, 2);
  await writeTextFile(filename, json, {
    baseDir: BaseDirectory.Download,
  });
}
```

### Rust-Side File Operations

```rust
// For operations that need direct file access (PNG metadata, large files)
use std::fs::File;
use std::io::Read;
use tauri::AppHandle;
use tauri_plugin_fs::FsExt;

pub fn read_character_card_png(
    app: &AppHandle,
    path: &std::path::Path,
) -> Result<Character, AppError> {
    // Add file to scope if needed
    let scope = app.fs_scope();
    scope.allow_file(path).map_err(|e| AppError::Io(e.to_string()))?;
    
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Parse PNG chunks...
    parse_png_metadata(&buffer)
}
```

---

## 7. Multi-Window Considerations

### Single-Window vs Multi-Window

**Recommendation for Persona:** Start with **single window**, add secondary windows later if needed.

```rust
// Window management
use tauri::{AppHandle, WebviewWindowBuilder, WebviewUrl};

// Creating a secondary window (e.g., for character editor)
pub fn open_character_editor(
    app: &AppHandle,
    character_id: Option<String>,
) -> Result<(), AppError> {
    let label = character_id
        .map(|id| format!("character-{}", id))
        .unwrap_or_else(|| "character-new".to_string());
    
    // Check if window already exists
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus()?;
        return Ok(());
    }
    
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App("/character".into()))
        .title("Character Editor")
        .inner_size(800.0, 600.0)
        .build()?;
    
    Ok(())
}
```

### Window-to-Window Communication

```rust
// Sending events to specific windows
use tauri::Emitter;

#[tauri::command]
fn update_character(
    app: AppHandle,
    character: Character,
) -> Result<(), AppError> {
    // Save to DB...
    
    // Notify main window of update
    app.emit_to("main", "character-updated", &character)?;
    
    Ok(())
}
```

### Single Instance Pattern

```toml
# Cargo.toml
[dependencies]
tauri-plugin-single-instance = "2"
```

```rust
// Prevent multiple app instances
.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
    // Focus existing window
    if let Some(window) = app.get_webview_window("main") {
        window.set_focus().unwrap();
    }
}))
```

---

## 8. Build and Distribution Architecture

### Development vs Production Configurations

```json
// tauri.conf.json (development defaults)
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi", "appimage"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "Productivity",
    "shortDescription": "AI roleplay application"
  }
}
```

### Cargo.toml Optimization

```toml
# src-tauri/Cargo.toml - Optimized for small binary size
[profile.dev]
incremental = true

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"     # Optimize for size
panic = "abort"
strip = true
```

### Distribution Targets

| Platform | Target | Format | Notes |
|----------|--------|--------|-------|
| Windows | x86_64 | .msi, .exe | Use `targets: ["msi"]` |
| macOS | aarch64, x86_64 | .app, .dmg | Universal binary with both |
| Linux | x86_64 | .AppImage, .deb | AppImage for portability |

### CI/CD Pipeline

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - uses: dtolnay/rust-action@stable
      
      - name: Install dependencies
        run: npm ci
      
      - name: Build Tauri
        run: npm run tauri build
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: binaries-${{ matrix.platform }}
          path: src-tauri/target/release/bundle
```

---

## Recommended Build Order

Based on dependencies between components:

### Phase 1: Foundation (Week 1)
1. **Project scaffold** - Tauri + React + TypeScript setup
2. **Database layer** - SQLite schema + migrations
3. **Core types** - Shared TypeScript/Rust types
4. **Basic IPC** - Hello world commands

### Phase 2: Core Features (Weeks 2-3)
1. **Character management** - CRUD + storage
2. **Chat infrastructure** - Messages, conversations
3. **AI integration** - HTTP client + streaming
4. **Settings** - Provider configuration

### Phase 3: Polish (Week 4)
1. **UI refinement** - GNOME design system
2. **Search** - FTS5 implementation
3. **Import/Export** - CharacterCardV2 format
4. **Distribution** - Build pipeline

### Dependency Graph

```
Project Setup
    ↓
Database Schema
    ↓
Rust Models + Types
    ↓
┌───────────┬───────────┐
↓           ↓           ↓
Character  Provider   Settings
Commands   Commands   Commands
    ↓           ↓           ↓
    └───────────┴───────────┘
                ↓
         Chat Commands
                ↓
         Frontend UI
                ↓
         Distribution
```

---

## Sources

- Tauri Official Documentation: https://tauri.app/
- Tauri IPC Concepts: https://tauri.app/concept/inter-process-communication/
- Tauri State Management: https://tauri.app/develop/state-management/
- Tauri SQL Plugin: https://tauri.app/plugin/sql/
- Tauri File System: https://tauri.app/plugin/file-system/
- Tauri Distribution: https://tauri.app/distribute/
- Tauri Store Plugin: https://tauri.app/plugin/store/

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| IPC Patterns | HIGH | Official Tauri docs + established patterns |
| Frontend Architecture | HIGH | Standard React patterns, well-documented |
| Rust Module Organization | HIGH | Based on Tauri examples + Rust best practices |
| SQLite Schema | HIGH | Standard chat app patterns + CharacterCardV2 spec |
| State Management | MEDIUM-HIGH | Tauri patterns + standard React patterns |
| File System | HIGH | Official plugin documentation |
| Multi-Window | MEDIUM | Tauri supports it, but less documentation |
| Build/Distribution | HIGH | Official Tauri docs comprehensive |

---

## Open Questions

1. **Vault Mode Encryption**: Research needed on encrypting SQLite database at rest
2. **Conversation Branching**: UI pattern for displaying branching conversations needs design
3. **Lorebook Keyword Matching**: Performance implications of runtime keyword scanning
4. **Multi-provider Concurrency**: How to handle multiple concurrent AI connections
