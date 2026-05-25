# RoleTect

**RoleTect** is a powerful, privacy-first web & native application designed to streamline the job application process. By combining a local-first architecture with cutting-edge AI, RoleTect allows you to effortlessly manage your job search, securely tailor your resumes and cover letters using LaTeX, and keep track of your application status—all without compromising your data.

---

## 🚀 Features

*   **Local-First Privacy:** All your data, resumes, and job tracking information are stored locally in a secure SQLite database on your machine.
*   **AI-Powered Tailoring:** Connect your preferred AI provider (Gemini, OpenAI, Anthropic, or Groq) to automatically extract job details and tailor your base LaTeX resumes and cover letters to specific job descriptions.
*   **Integrated LaTeX Compiler:** Edit your templates and immediately preview the generated PDFs thanks to the embedded Tectonic LaTeX engine. No external dependencies required.
*   **Browser Extension Integration:** Seamlessly capture job postings directly from your browser (Chrome/Firefox) and send them straight to your RoleTect inbox with a single click.
*   **Application Tracking:** Track the status of every application from drafting to offer, including dates, salary details, and custom notes.
*   **Theme Support:** Customize your workspace with built-in themes or create your own using the integrated theme editor.

---

## 👨‍💻 For Users (Job Seekers)

RoleTect is your personal command center for landing your next role.

### Getting Started

1.  **Download the App:** Pre-compiled binaries for Windows, macOS, and Linux are available in the [Releases](https://github.com/AhmedTrooper/RoleTect/releases) section of this repository. Download the installer for your operating system.
2.  **Set Up AI:** Go to the **Settings** tab and enter the API key for your preferred AI provider (e.g., Google Gemini). Your key is securely encrypted locally.
3.  **Install the Extension (Optional but Recommended):** Install the companion browser extension for Chrome or Firefox from the `extentions/` folder. This allows you to quickly import jobs into your RoleTect Inbox.
4.  **Create Base Templates:** Navigate to the **Resumes** and **Cover Letters** tabs to set up your master LaTeX templates.

### Workflow

1.  **Ingest:** Find a job you like online and use the browser extension to send it to RoleTect.
2.  **Parse:** Open the **Inbox** in RoleTect, review the job, and let the AI parse the requirements and responsibilities.
3.  **Tailor:** Once the job is saved, generate a tailored version of your resume or cover letter. The AI will intelligently adjust the content of your LaTeX template to highlight the most relevant skills.
4.  **Compile & Apply:** Review the generated document in the **Compiler**, download the PDF, and submit your application.
5.  **Track:** Update the job's status (Applied, Interviewing, Offer) to keep your search organized.

---

## 🎯 For Tech Recruiters

RoleTect isn't just for job seekers. If you manage multiple candidates or help structure resumes, RoleTect offers a robust platform for document generation and management.

*   **Standardized Formatting:** Use the LaTeX compiler to ensure all candidate profiles adhere to a consistent, professional layout before presenting them to clients.
*   **Rapid Tailoring:** Quickly adjust a candidate's master profile to highlight specific skills required for a particular requisition using the AI tailoring tools.
*   **Data Security:** Because RoleTect is local-first, you maintain strict confidentiality and compliance regarding candidate data. No candidate information is stored on external application servers.

---

## 🛠️ For Developers

RoleTect is built with **Tauri v2**, a **Vue 3** frontend, and a **Rust** backend.

### Prerequisites

Ensure you have the following installed:
*   [Node.js](https://nodejs.org/) (v18+)
*   [Rust](https://www.rust-lang.org/tools/install)
*   [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) (C++ build tools, WebKit, etc., depending on your OS)
*   [Bun](https://bun.sh/) (Optional, but used in the project configuration)

### Local Setup

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/AhmedTrooper/RoleTect.git
    cd RoleTect
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    # or
    bun install
    ```

3.  **Run the development server:**
    This command will start the Vite development server and open the Tauri desktop window.
    ```bash
    npm run tauri dev
    ```

### Architecture Overview

*   **`src/`:** The Vue 3 frontend application. Manages UI, routing, and state (Pinia).
*   **`src-tauri/src/`:** The Rust backend.
    *   `db.rs`: SQLite database schema and migrations.
    *   `ai.rs`: Integrations with AI providers via the `rig` library.
    *   `server.rs`: A local Axum server that receives job data from the browser extensions.
    *   `commands/`: Tauri commands invoked by the frontend.
*   **`extentions/`:** Source code for the Chrome and Firefox browser extensions.

### Building for Production

To compile the application into a standalone binary:

```bash
npm run tauri build
```
The compiled binaries will be located in `src-tauri/target/release/bundle/`.
