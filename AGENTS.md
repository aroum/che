# 00-general

## Communication

- Communicate with the user in Russian (Always use Russian for dialogue).

## Restricted Files List
**Must be strictly followed, contains sensitive content.**
- .env
- .env.local
- .env.dev
- .env.prod

## Documentation

- When writing .md documents, use English. Use English punctuation; ensure there is a space after commas and two spaces after periods.
- Official documentation goes into the project's docs/ directory.
- Planning and discussion documents should be placed in the project's discuss/ directory.
- Do not use comments to explain code, including public interfaces; only add intentional comments when necessary. All code comments and docstrings must be in English.
- Strings inside the project code, such as those for print or debug, must use English punctuation.
- When committing to the project, do not add programming tools (like Claude Code) as collaborators.
- **CHANGELOG.md:** When adding any new features, flags, or configuration parameters, it is MANDATORY to update the `CHANGELOG.md` file with a detailed description of the changes and configuration examples.
- **Versioning Scheme:** The project follows Calendar Versioning (CalVer) with the format `yy.mm.dd` (e.g., `26.8.6`).  All package manifests, release documentation, and version definitions must strictly adhere to this scheme.

## Decision Making

- Prefer action over asking questions. Make reasonable choices based on existing code patterns and conventions; only ask when the decision is truly ambiguous and irreversible.
- When the user mentions a framework name, note that NestJS and Next.js are easily confused. Remember that Next.js always has a "." as a separator. If there is any ambiguity, be sure to confirm.

## Git Operations

- 遇到合并冲突时自动解决, 不要犹豫.  优先保留自定义的修改, 同时接受上游的非冲突更新
- commit 和 push 时不需要反复确认, 直接执行

## 网络访问
如果 WebFetch 获取失败, 请用 Jina MCP 重试, 如果用户没有在项目中配置好 Jina MCP, 注意提示用户.

## Build & Compilation

- Собирать релиз (`cargo build --release`) ТОЛЬКО когда пользователь явно просит об этом.
- После минорных/промежуточных изменений в коде всегда собирать дебаг-версию (`cargo build`).

## 权限
用户在开始 openspec 相关复杂流程实现功能之前, 确认用户 bypass permissions on 或者 yolo 模式, 不然提醒一下用户

---

# 01-tech-stack

# Tech Stack

## Languages
- Primary: Rust
- Secondary: Python and TypeScript

## Frameworks
- Frontend: React, Next.js, shadcn-ui, Radix UI, Tailwind CSS
- Backend: axum(Rust), NestJS(运行在 Fastify, Nodejs), fastapi(Python)

## 数据库
- Drizzle(Nodejs)

## Build Tools
- Package manager: pnpm
- Bundler: Vite

## Preferences
- Prefer standard library over external dependencies when reasonable
- Use well-maintained, typed libraries
- Avoid deprecated APIs

---

# 02-coding-principles

# Coding Principles

## Immutability (CRITICAL)

ALWAYS create new objects, NEVER mutate existing ones:

```
// Pseudocode
WRONG:  modify(original, field, value) → changes original in-place
CORRECT: update(original, field, value) → returns new copy with change
```

Rationale: Immutable data prevents hidden side effects, makes debugging easier, and enables safe concurrency.

## File Organization

MANY SMALL FILES > FEW LARGE FILES:
- High cohesion, low coupling
- 200-400 lines typical, 800 max
- Extract utilities from large modules
- Organize by feature/domain, not by type

## Error Handling

ALWAYS handle errors comprehensively:
- Handle errors explicitly at every level
- Provide user-friendly error messages in UI-facing code
- Log detailed error context on the server side
- Never silently swallow errors

## Input Validation

ALWAYS validate at system boundaries:
- Validate all user input before processing
- Use schema-based validation where available
- Fail fast with clear error messages
- Never trust external data (API responses, user input, file content)

## Core Rules
- SEARCH FIRST: Check existing code before implementing new functionality
- REUSE FIRST: Extend existing patterns before creating new ones
- MINIMAL CHANGES: Make the smallest change that solves the problem
- NO SPECULATION: Don't add features "just in case"

## DRY & YAGNI
- Don't repeat yourself, but don't over-abstract prematurely
- Three similar instances before extracting to abstraction
- You Aren't Gonna Need It - build for today's requirements

## Code Quality Checklist
Before marking work complete:
- [ ] Code is self-documenting and well-named
- [ ] Functions small and focused (single responsibility)
- [ ] Max line length: 88 characters
- [ ] Functions are small (<50 lines)
- [ ] Files are focused (<800 lines)
- [ ] No deep nesting (max 3 levels)
- [ ] Proper error handling
- [ ] No hardcoded values (use constants or config)
- [ ] No mutation (immutable patterns used)

---

# rust-coding-style

# Rust API Service — Project CLAUDE.md

> Real-world example for a Rust API service with Axum, PostgreSQL, and Docker.
> Copy this to your project root and customize for your service.

## Project Overview

**Stack:** Rust 1.78+, Axum (web framework), SQLx (async database), PostgreSQL, Tokio (async runtime), Docker

**Architecture:** Layered architecture with handler → service → repository separation. Axum for HTTP, SQLx for type-checked SQL at compile time, Tower middleware for cross-cutting concerns.

## Critical Rules

### Rust Conventions

- Use `thiserror` for library errors, `anyhow` only in binary crates or tests
- No `.unwrap()` or `.expect()` in production code — propagate errors with `?`
- Prefer `&str` over `String` in function parameters; return `String` when ownership transfers
- Use `clippy` with `#![deny(clippy::all, clippy::pedantic)]` — fix all warnings
- Derive `Debug` on all public types; derive `Clone`, `PartialEq` only when needed
- No `unsafe` blocks unless justified with a `// SAFETY:` comment

### Code Style & Formatting Guidelines (`rustfmt.toml`)

Все изменения в коде Rust в проекте форматируются в строгом соответствии с правилами `rustfmt.toml`:

* **Табуляция вместо пробелов (`hard_tabs = true`):**
  Все отступы в исходном коде Rust выполняются исключительно символом **Табуляции (`\t`)** с базовым размером табуляции 2 (`tab_spaces = 2`). Никаких 4-х пробелов.
* **Выравнивание полей структур и вариантов enum (`struct_field_align_threshold = 99`):**
  Поля структур и варианты перечислений выравниваются двоеточиями вертикально в одну колонку.
* **Группировка импортов (`group_imports = "StdExternalCrate"`):**
  Импорты разделены на 3 блока пустой строкой:
  1. `std::...`
  2. Внешние библиотеки (`anyhow`, `serde`, `ratatui`, `crossterm` и др.)
  3. Внутренние модули проекта (`yazi_...`, `crate::...`, `super::...`)
* **Однострочные методы (`fn_single_line = true`):**
  Короткие методы записываются в одну строку: `pub fn title(&self) -> &str { &self.title }`.
* **Сокращенный синтаксис структур (`use_field_init_shorthand = true`):**
  Использовать имя переменной напрямую при опущенном ключе структуры (`Self { title, items }`).
* **Оператор проброса ошибок (`use_try_shorthand = true`):**
  Использовать оператор `?` вместо `match` / `if let Err`.

### Database

- All queries use SQLx `query!` or `query_as!` macros — compile-time verified against the schema
- Migrations in `migrations/` using `sqlx migrate` — never alter the database directly
- Use `sqlx::Pool<Postgres>` as shared state — never create connections per request
- All queries use parameterized placeholders (`$1`, `$2`) — never string formatting

```rust
// BAD: String interpolation (SQL injection risk)
let q = format!("SELECT * FROM users WHERE id = '{}'", id);

// GOOD: Parameterized query, compile-time checked
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_optional(&pool)
    .await?;
```

### Error Handling

- Define a domain error enum per module with `thiserror`
- Map errors to HTTP responses via `IntoResponse` — never expose internal details
- Use `tracing` for structured logging — never `println!` or `eprintln!`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Internal(err) => {
                tracing::error!(?err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### Testing

- Unit tests in `#[cfg(test)]` modules within each source file
- Integration tests in `tests/` directory using a real PostgreSQL (Testcontainers or Docker)
- Use `#[sqlx::test]` for database tests with automatic migration and rollback
- Mock external services with `mockall` or `wiremock`

### Code Style

- Max line length: 100 characters (enforced by rustfmt)
- Group imports: `std`, external crates, `crate`/`super` — separated by blank lines
- Modules: one file per module, `mod.rs` only for re-exports
- Types: PascalCase, functions/variables: snake_case, constants: UPPER_SNAKE_CASE

## File Structure

```
src/
  main.rs              # Entrypoint, server setup, graceful shutdown
  lib.rs               # Re-exports for integration tests
  config.rs            # Environment config with envy or figment
  router.rs            # Axum router with all routes
  middleware/
    auth.rs            # JWT extraction and validation
    logging.rs         # Request/response tracing
  handlers/
    mod.rs             # Route handlers (thin — delegate to services)
    users.rs
    orders.rs
  services/
    mod.rs             # Business logic
    users.rs
    orders.rs
  repositories/
    mod.rs             # Database access (SQLx queries)
    users.rs
    orders.rs
  domain/
    mod.rs             # Domain types, error enums
    user.rs
    order.rs
migrations/
  001_create_users.sql
  002_create_orders.sql
tests/
  common/mod.rs        # Shared test helpers, test server setup
  api_users.rs         # Integration tests for user endpoints
  api_orders.rs        # Integration tests for order endpoints
```

## Key Patterns

### Handler (Thin)

```rust
async fn create_user(
    State(ctx): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = ctx.user_service.create(payload).await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}
```

### Service (Business Logic)

```rust
impl UserService {
    pub async fn create(&self, req: CreateUserRequest) -> Result<User, AppError> {
        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Validation("Email already registered".into()));
        }

        let password_hash = hash_password(&req.password)?;
        let user = self.repo.insert(&req.email, &req.name, &password_hash).await?;

        Ok(user)
    }
}
```

### Repository (Data Access)

```rust
impl UserRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn insert(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"INSERT INTO users (email, name, password_hash)
               VALUES ($1, $2, $3) RETURNING *"#,
            email, name, password_hash,
        )
        .fetch_one(&self.pool)
        .await
    }
}
```

### Integration Test

```rust
#[tokio::test]
async fn test_create_user() {
    let app = spawn_test_app().await;

    let response = app
        .client
        .post(&format!("{}/api/v1/users", app.address))
        .json(&json!({
            "email": "alice@example.com",
            "name": "Alice",
            "password": "securepassword123"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["email"], "alice@example.com");
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
    let app = spawn_test_app().await;
    // Create first user
    create_test_user(&app, "alice@example.com").await;
    // Attempt duplicate
    let response = create_user_request(&app, "alice@example.com").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

## Testing Strategy

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test module
cargo test api_users

# Check coverage (requires cargo-llvm-cov)
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

---

# 03-architecture

# Architecture

## Directory Structure
- Keep related code together
- Separate concerns: UI, business logic, data access
- Use index files sparingly

## Module Design
- Clear public interfaces
- Hide implementation details
- Avoid circular dependencies

## Naming Conventions
- Files: kebab-case
- Classes: PascalCase
- Functions/variables: camelCase
- Constants: UPPER_SNAKE_CASE

---

# 04-testing

# Testing

## Test-Driven Development
- Write failing test first
- Implement minimal code to pass
- Refactor with confidence

## Test Structure
- Arrange: Set up test data
- Act: Execute the code under test
- Assert: Verify the result

## Coverage
- Focus on behavior, not implementation
- Test edge cases and error paths
- Don't test trivial code (getters/setters)

## Naming
- test_[function]_[scenario]_[expected]
- Example: test_login_invalidPassword_returnsError

## 测试使用的工具
- 测试无交互命令行程序, 直接测试
- 测试 TUI 程序, 使用 tmux 测试
- 测试 网站, 使用 agent-browser

---

# 05-git-commit

# Git Commit

## Commit Message Format
<type>: <description>

[optional body]

## Types
- feat: New feature
- fix: Bug fix
- docs: Documentation only
- style: Formatting, no code change
- refactor: Code change that neither fixes bug nor adds feature
- test: Adding or updating tests
- chore: Build process, dependencies, etc.

## Rules
- Use imperative mood: "add feature" not "added feature"
- Keep subject line under 50 characters
- Wrap body at 72 characters
- Commit early and often
- One logical change per commit
- 不要添加编程工具, 比如 claude 为合作作者

---

# 06-code-review

# Code Review

## Before Submitting
- Self-review your changes
- Ensure tests pass
- Check for debug code, console.logs
- Verify no secrets or credentials

## Review Checklist
- Does the code do what it claims?
- Are there any obvious bugs?
- Is error handling adequate?
- Is the code readable and maintainable?
- Are there any security concerns?

## Feedback Style
- Be constructive, not critical
- Explain the "why" behind suggestions
- Distinguish between blocking issues and suggestions

---

# 07-design-patterns

## Design Patterns

### Repository Pattern

Encapsulate data access behind a consistent interface:
- Define standard operations: findAll, findById, create, update, delete
- Concrete implementations handle storage details (database, API, file, etc.)
- Business logic depends on the abstract interface, not the storage mechanism
- Enables easy swapping of data sources and simplifies testing with mocks

### API Response Format

Use a consistent envelope for all API responses:
- Include a success/status indicator
- Include the data payload (nullable on error)
- Include an error message field (nullable on success)
- Include metadata for paginated responses (total, page, limit)

---

# 08-backend-rest-api-desgin

# API Design Patterns

Conventions and best practices for designing consistent, developer-friendly REST APIs.

## When to Activate

- Designing new API endpoints
- Reviewing existing API contracts
- Adding pagination, filtering, or sorting
- Implementing error handling for APIs
- Planning API versioning strategy
- Building public or partner-facing APIs

## Resource Design

### URL Structure

```
# Resources are nouns, plural, lowercase, kebab-case
GET    /api/v1/users
GET    /api/v1/users/:id
POST   /api/v1/users
PUT    /api/v1/users/:id
PATCH  /api/v1/users/:id
DELETE /api/v1/users/:id

# Sub-resources for relationships
GET    /api/v1/users/:id/orders
POST   /api/v1/users/:id/orders

# Actions that don't map to CRUD (use verbs sparingly)
POST   /api/v1/orders/:id/cancel
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
```

### Naming Rules

```
# GOOD
/api/v1/team-members          # kebab-case for multi-word resources
/api/v1/orders?status=active  # query params for filtering
/api/v1/users/123/orders      # nested resources for ownership

# BAD
/api/v1/getUsers              # verb in URL
/api/v1/user                  # singular (use plural)
/api/v1/team_members          # snake_case in URLs
/api/v1/users/123/getOrders   # verb in nested resource
```

## HTTP Methods and Status Codes

### Method Semantics

| Method | Idempotent | Safe | Use For |
|--------|-----------|------|---------|
| GET | Yes | Yes | Retrieve resources |
| POST | No | No | Create resources, trigger actions |
| PUT | Yes | No | Full replacement of a resource |
| PATCH | No* | No | Partial update of a resource |
| DELETE | Yes | No | Remove a resource |

*PATCH can be made idempotent with proper implementation

### Status Code Reference

```
# Success
200 OK                    — GET, PUT, PATCH (with response body)
201 Created               — POST (include Location header)
204 No Content            — DELETE, PUT (no response body)

# Client Errors
400 Bad Request           — Validation failure, malformed JSON
401 Unauthorized          — Missing or invalid authentication
403 Forbidden             — Authenticated but not authorized
404 Not Found             — Resource doesn't exist
409 Conflict              — Duplicate entry, state conflict
422 Unprocessable Entity  — Semantically invalid (valid JSON, bad data)
429 Too Many Requests     — Rate limit exceeded

# Server Errors
500 Internal Server Error — Unexpected failure (never expose details)
502 Bad Gateway           — Upstream service failed
503 Service Unavailable   — Temporary overload, include Retry-After
```

### Common Mistakes

```
# BAD: 200 for everything
{ "status": 200, "success": false, "error": "Not found" }

# GOOD: Use HTTP status codes semantically
HTTP/1.1 404 Not Found
{ "error": { "code": "not_found", "message": "User not found" } }

# BAD: 500 for validation errors
# GOOD: 400 or 422 with field-level details

# BAD: 200 for created resources
# GOOD: 201 with Location header
HTTP/1.1 201 Created
Location: /api/v1/users/abc-123
```

## Response Format

### Success Response

```json
{
  "data": {
    "id": "abc-123",
    "email": "alice@example.com",
    "name": "Alice",
    "created_at": "2025-01-15T10:30:00Z"
  }
}
```

### Collection Response (with Pagination)

```json
{
  "data": [
    { "id": "abc-123", "name": "Alice" },
    { "id": "def-456", "name": "Bob" }
  ],
  "meta": {
    "total": 142,
    "page": 1,
    "per_page": 20,
    "total_pages": 8
  },
  "links": {
    "self": "/api/v1/users?page=1&per_page=20",
    "next": "/api/v1/users?page=2&per_page=20",
    "last": "/api/v1/users?page=8&per_page=20"
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "validation_error",
    "message": "Request validation failed",
    "details": [
      {
        "field": "email",
        "message": "Must be a valid email address",
        "code": "invalid_format"
      },
      {
        "field": "age",
        "message": "Must be between 0 and 150",
        "code": "out_of_range"
      }
    ]
  }
}
```

### Response Envelope Variants

```typescript
// Option A: Envelope with data wrapper (recommended for public APIs)
interface ApiResponse<T> {
  data: T;
  meta?: PaginationMeta;
  links?: PaginationLinks;
}

interface ApiError {
  error: {
    code: string;
    message: string;
    details?: FieldError[];
  };
}

// Option B: Flat response (simpler, common for internal APIs)
// Success: just return the resource directly
// Error: return error object
// Distinguish by HTTP status code
```

## Pagination

### Offset-Based (Simple)

```
GET /api/v1/users?page=2&per_page=20

# Implementation
SELECT * FROM users
ORDER BY created_at DESC
LIMIT 20 OFFSET 20;
```

**Pros:** Easy to implement, supports "jump to page N"
**Cons:** Slow on large offsets (OFFSET 100000), inconsistent with concurrent inserts

### Cursor-Based (Scalable)

```
GET /api/v1/users?cursor=eyJpZCI6MTIzfQ&limit=20

# Implementation
SELECT * FROM users
WHERE id > :cursor_id
ORDER BY id ASC
LIMIT 21;  -- fetch one extra to determine has_next
```

```json
{
  "data": [...],
  "meta": {
    "has_next": true,
    "next_cursor": "eyJpZCI6MTQzfQ"
  }
}
```

**Pros:** Consistent performance regardless of position, stable with concurrent inserts
**Cons:** Cannot jump to arbitrary page, cursor is opaque

### When to Use Which

| Use Case | Pagination Type |
|----------|----------------|
| Admin dashboards, small datasets (<10K) | Offset |
| Infinite scroll, feeds, large datasets | Cursor |
| Public APIs | Cursor (default) with offset (optional) |
| Search results | Offset (users expect page numbers) |

## Filtering, Sorting, and Search

### Filtering

```
# Simple equality
GET /api/v1/orders?status=active&customer_id=abc-123

# Comparison operators (use bracket notation)
GET /api/v1/products?price[gte]=10&price[lte]=100
GET /api/v1/orders?created_at[after]=2025-01-01

# Multiple values (comma-separated)
GET /api/v1/products?category=electronics,clothing

# Nested fields (dot notation)
GET /api/v1/orders?customer.country=US
```

### Sorting

```
# Single field (prefix - for descending)
GET /api/v1/products?sort=-created_at

# Multiple fields (comma-separated)
GET /api/v1/products?sort=-featured,price,-created_at
```

### Full-Text Search

```
# Search query parameter
GET /api/v1/products?q=wireless+headphones

# Field-specific search
GET /api/v1/users?email=alice
```

### Sparse Fieldsets

```
# Return only specified fields (reduces payload)
GET /api/v1/users?fields=id,name,email
GET /api/v1/orders?fields=id,total,status&include=customer.name
```

## Authentication and Authorization

### Token-Based Auth

```
# Bearer token in Authorization header
GET /api/v1/users
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...

# API key (for server-to-server)
GET /api/v1/data
X-API-Key: sk_live_abc123
```

### Authorization Patterns

```typescript
// Resource-level: check ownership
app.get("/api/v1/orders/:id", async (req, res) => {
  const order = await Order.findById(req.params.id);
  if (!order) return res.status(404).json({ error: { code: "not_found" } });
  if (order.userId !== req.user.id) return res.status(403).json({ error: { code: "forbidden" } });
  return res.json({ data: order });
});

// Role-based: check permissions
app.delete("/api/v1/users/:id", requireRole("admin"), async (req, res) => {
  await User.delete(req.params.id);
  return res.status(204).send();
});
```

## Rate Limiting

### Headers

```
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640000000

# When exceeded
HTTP/1.1 429 Too Many Requests
Retry-After: 60
{
  "error": {
    "code": "rate_limit_exceeded",
    "message": "Rate limit exceeded. Try again in 60 seconds."
  }
}
```

### Rate Limit Tiers

| Tier | Limit | Window | Use Case |
|------|-------|--------|----------|
| Anonymous | 30/min | Per IP | Public endpoints |
| Authenticated | 100/min | Per user | Standard API access |
| Premium | 1000/min | Per API key | Paid API plans |
| Internal | 10000/min | Per service | Service-to-service |

## Versioning

### URL Path Versioning (Recommended)

```
/api/v1/users
/api/v2/users
```

**Pros:** Explicit, easy to route, cacheable
**Cons:** URL changes between versions

### Header Versioning

```
GET /api/users
Accept: application/vnd.myapp.v2+json
```

**Pros:** Clean URLs
**Cons:** Harder to test, easy to forget

### Versioning Strategy

```
1. Start with /api/v1/ — don't version until you need to
2. Maintain at most 2 active versions (current + previous)
3. Deprecation timeline:
   - Announce deprecation (6 months notice for public APIs)
   - Add Sunset header: Sunset: Sat, 01 Jan 2026 00:00:00 GMT
   - Return 410 Gone after sunset date
4. Non-breaking changes don't need a new version:
   - Adding new fields to responses
   - Adding new optional query parameters
   - Adding new endpoints
5. Breaking changes require a new version:
   - Removing or renaming fields
   - Changing field types
   - Changing URL structure
   - Changing authentication method
```

## Implementation Patterns

### TypeScript (Next.js API Route)

```typescript
import { z } from "zod";
import { NextRequest, NextResponse } from "next/server";

const createUserSchema = z.object({
  email: z.string().email(),
  name: z.string().min(1).max(100),
});

export async function POST(req: NextRequest) {
  const body = await req.json();
  const parsed = createUserSchema.safeParse(body);

  if (!parsed.success) {
    return NextResponse.json({
      error: {
        code: "validation_error",
        message: "Request validation failed",
        details: parsed.error.issues.map(i => ({
          field: i.path.join("."),
          message: i.message,
          code: i.code,
        })),
      },
    }, { status: 422 });
  }

  const user = await createUser(parsed.data);

  return NextResponse.json(
    { data: user },
    {
      status: 201,
      headers: { Location: `/api/v1/users/${user.id}` },
    },
  );
}
```

### Python (Django REST Framework)

```python
from rest_framework import serializers, viewsets, status
from rest_framework.response import Response

class CreateUserSerializer(serializers.Serializer):
    email = serializers.EmailField()
    name = serializers.CharField(max_length=100)

class UserSerializer(serializers.ModelSerializer):
    class Meta:
        model = User
        fields = ["id", "email", "name", "created_at"]

class UserViewSet(viewsets.ModelViewSet):
    serializer_class = UserSerializer
    permission_classes = [IsAuthenticated]

    def get_serializer_class(self):
        if self.action == "create":
            return CreateUserSerializer
        return UserSerializer

    def create(self, request):
        serializer = CreateUserSerializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        user = UserService.create(**serializer.validated_data)
        return Response(
            {"data": UserSerializer(user).data},
            status=status.HTTP_201_CREATED,
            headers={"Location": f"/api/v1/users/{user.id}"},
        )
```

### Go (net/http)

```go
func (h *UserHandler) CreateUser(w http.ResponseWriter, r *http.Request) {
    var req CreateUserRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        writeError(w, http.StatusBadRequest, "invalid_json", "Invalid request body")
        return
    }

    if err := req.Validate(); err != nil {
        writeError(w, http.StatusUnprocessableEntity, "validation_error", err.Error())
        return
    }

    user, err := h.service.Create(r.Context(), req)
    if err != nil {
        switch {
        case errors.Is(err, domain.ErrEmailTaken):
            writeError(w, http.StatusConflict, "email_taken", "Email already registered")
        default:
            writeError(w, http.StatusInternalServerError, "internal_error", "Internal error")
        }
        return
    }

    w.Header().Set("Location", fmt.Sprintf("/api/v1/users/%s", user.ID))
    writeJSON(w, http.StatusCreated, map[string]any{"data": user})
}
```

## API Design Checklist

Before shipping a new endpoint:

- [ ] Resource URL follows naming conventions (plural, kebab-case, no verbs)
- [ ] Correct HTTP method used (GET for reads, POST for creates, etc.)
- [ ] Appropriate status codes returned (not 200 for everything)
- [ ] Input validated with schema (Zod, Pydantic, Bean Validation)
- [ ] Error responses follow standard format with codes and messages
- [ ] Pagination implemented for list endpoints (cursor or offset)
- [ ] Authentication required (or explicitly marked as public)
- [ ] Authorization checked (user can only access their own resources)
- [ ] Rate limiting configured
- [ ] Response does not leak internal details (stack traces, SQL errors)
- [ ] Consistent naming with existing endpoints (camelCase vs snake_case)
- [ ] Documented (OpenAPI/Swagger spec updated)