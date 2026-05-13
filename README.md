# CVSynth

**CVSynth** is a high-precision, desktop-native resume tailoring engine. It leverages the power of Rust, Tauri, and AI to transform raw job descriptions into structured data and generate targeted LaTeX resumes with absolute control.

![License](https://img.shields.io/badge/license-MIT-00e599?style=flat-square)
![Tauri](https://img.shields.io/badge/Tauri-2.0-00e599?style=flat-square&logo=tauri)
![Rust](https://img.shields.io/badge/Rust-2021-00e599?style=flat-square&logo=rust)
![Bun](https://img.shields.io/badge/Bun-1.1-00e599?style=flat-square&logo=bun)

---

## ✨ Features

* **Multi-Provider AI Intelligence:** Swap between OpenAI, Google Gemini, and Groq seamlessly via the [Rig](https://github.com/0xPlayground/rig) library.
* **Military-Grade Security:** Sensitive API credentials are encrypted locally using **AES-256-GCM** via [Tauri Stronghold](https://github.com/tauri-apps/tauri-plugin-stronghold).
* **Persistent Pipeline:** Manage your job application lifecycle with a local SQLite database for speed and offline access.
* **Premium Developer Experience:** A sleek, pitch-black UI featuring a neon-emerald grid aesthetic, designed for high-focus productivity.
* **Slug-Based Routing:** Secure, unique URL-friendly identifiers (`nanoid`) for every job and resume.
* **LaTeX Precision:** Forces AI to output valid LaTeX code using custom XML-tag wrapping for 100% clean document generation.

---

## 🛠️ Tech Stack

* **Frontend:** Vue 3 (Composition API), Pinia (Modular Stores), Vue Router.
* **Backend:** Rust, Tauri v2.
* **Database:** SQLite (`rusqlite`) with custom triggers for `updated_at` tracking and `CHECK` constraints for enums.
* **AI Orchestration:** Rig Core.
* **Package Manager:** Bun.

---

## 🚀 Getting Started

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install)
* [Bun](https://bun.sh/)
* [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites)

### Installation

1.  **Clone the repository**
    ```bash
    git clone [https://github.com/ahmedtrooper/cvsynth.git](https://github.com/ahmedtrooper/cvsynth.git)
    cd cvsynth
    ```

2.  **Install dependencies**
    ```bash
    bun install
    ```

3.  **Run in development mode**
    ```bash
    bun tauri dev
    ```

---

## 📁 Architecture

The project follows a strict **Feature-Based Modular Architecture**:

* `/src/store/`: Modular Pinia stores (`settings.ts`, `jobs.ts`, `resumes.ts`).
* `/src-tauri/src/commands/`: Feature-isolated Rust commands for settings and job management.
* `/src-tauri/src/db.rs`: Centralized SQLite schema management with secure `TEXT PRIMARY KEY` slug implementation.
* `/src-tauri/src/ai.rs`: Abstracted AI logic for multi-provider support and structured JSON extraction.

---

## 🔒 Security

CVSynth takes privacy seriously:
* **Zero Cloud Storage:** Your data and LaTeX templates stay on your machine.
* **No Plaintext Keys:** API keys are never stored in SQLite or environment variables; they are managed by the OS-integrated Stronghold vault.
* **Argon2 Hashing:** Stronghold vaults are protected using Argon2 password hashing with automated salt management.

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

---

**Built with 💚 and Absolute Control.**