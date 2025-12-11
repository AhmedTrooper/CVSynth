# NexusAI - Autonomous Project Orchestration System

NexusAI is a production-grade, full-stack monorepo designed to demonstrate the future of AI-native project management. It leverages autonomous agents to decompose high-level goals into actionable database tasks, synchronized in real-time across teams.

## 🏗 Tech Stack (The "Nexus" Stack)

| Layer          | Technology                         | Purpose                                                       |
| :------------- | :--------------------------------- | :------------------------------------------------------------ |
| **Monorepo**   | **Turborepo + Bun**                | High-performance workspace management.                        |
| **Frontend**   | **Next.js 15 (App Router)**        | Server Components, Suspense, and Actions.                     |
| **State**      | **Redux Toolkit + TanStack Query** | Separation of Client UI State vs. Server State.               |
| **UI/UX**      | **Shadcn UI + Framer Motion**      | Accessible components with production-grade animations.       |
| **Backend**    | **Nest.js**                        | Modular, event-driven architecture with Dependency Injection. |
| **AI Engine**  | **LangGraph.js**                   | Cyclic stateful agent workflows (Planner/Critic/Executor).    |
| **Database**   | **PostgreSQL + Prisma**            | Type-safe relational data.                                    |
| **Validation** | **Zod**                            | End-to-End type safety sharing schemas between Front/Back.    |

## 🚀 Getting Started

### Prerequisites

- **Bun** (v1.0+)
- **Docker** (For local Database & Redis)

### Installation

1. **Install Dependencies**
   ```bash
   bun install
   ```
