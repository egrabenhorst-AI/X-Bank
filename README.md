# X-Bank 🏦

![X-Bank Logo](https://via.placeholder.com/150?text=X-Bank)  
*A modern, microservices-based banking system built with Rust and Docker.*

---

## Overview

X-Bank is a cutting-edge banking application designed to demonstrate the power of microservices architecture. Built with **Rust** for performance and safety, and orchestrated with **Docker**, X-Bank provides a scalable, modular solution for managing accounts, transactions, authentication, universal basic income (UBI) distribution, auditing, and a sleek frontend.

### Key Features
- **Authentication Service**: Secure JWT-based user login.
- **Account Service**: Manage user accounts and balances.
- **Transaction Service**: Handle deposits and withdrawals.
- **UBI Service**: Distribute universal basic income to accounts.
- **Audit Service**: Log and secure all actions with SHA-256 hashing.
- **Frontend Service**: Simple, templated UI with Tera.

---

## Tech Stack

| Component         | Technology         |
|-------------------|--------------------|
| Language          | Rust 1.81          |
| Web Framework     | Actix-Web          |
| Database          | PostgreSQL (services), SQLite (audit) |
| Containerization  | Docker & Docker Compose |
| Templating        | Tera (frontend)    |
| Authentication    | JWT (jsonwebtoken) |
| Hashing           | SHA-256 (audit)    |

---

## Architecture

X-Bank follows a microservices pattern, with each service running in its own Docker container, communicating via HTTP endpoints. PostgreSQL handles core banking data, while SQLite logs audit trails.

```plaintext
+----------------+       +----------------+       +----------------+
| Auth Service   |<----->| Account Service|<----->| Transaction    |
| (JWT)          |       | (PostgreSQL)   |       | Service        |
+----------------+       +----------------+       +----------------+
       |                        |                        |
       v                        v                        v
+----------------+       +----------------+       +----------------+
| UBI Service    |<----->| Audit Service  |       | Frontend       |
| (Distributes)  |       | (SQLite, SHA256)|       | (Tera)         |
+----------------+       +----------------+       +----------------+
          All services orchestrated with Docker Compose
