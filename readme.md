<p align="center">
  <img src="./assets/bott.png"/>
</p>

# Welcome to `bott`: Your Terminal Copilot
`bott` (short for bot-in-terminal) is not just a command line tool; it's your copilot in the vast world of the terminal. Designed to make you feel like a terminal pro, bott assists you with day-to-day activities, provides helpful tips, and even adds a touch of humor to your command line experience.
<p align="left">
  <img src="./assets/query.gif"/>
</p>
<p align="right">
  <img src="./assets/debug.gif"/>
</p>

## Installation
### Rust
Install rust if you don't have it already and ensure a version greater than 1.74.0
```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ source $HOME/.cargo/env
$ rustc --version
```
### Bott
Install bott with a single command:
```bash
$ curl -o- https://raw.githubusercontent.com/subbu963/bott/v0.1.0/install.sh | bash 
```

## Usage
Whether you prefer the intelligence of OpenAI or the wisdom of Ollama, bott has you covered. Choose your Large Language Model (LLM) and unleash the power of your terminal.
### Choosing your LLM (Large Language Model)
#### With Openai
1. Navigate to [OpenAI's API Keys section](https://platform.openai.com/api-keys) and create a new API key or use an existing one.
2. Configure bott to use OpenAI:
```bash
$ bott! config set -k llm openai
$ bott! config set -k openai:api_key -v YOUR_API_KEY
```
Securely stored in a keychain, your API key is safe with bott.
3. Default model is `gpt-4`. Set your preferred OpenAI model (refer to [OpenAI's documentation](https://platform.openai.com/docs/models/gpt-4-and-gpt-4-turbo) for available models):
```bash
$ bott! config set -k openai:model -v YOUR_PREFERRED_MODEL
```
#### With Ollama
1. Download Ollama from [ollama.ai](https://ollama.ai).
2. Default model is `codellama:7b-instruct`. Choose your desired model (refer to the [library](https://ollama.ai/library) for available models), with a preference for codellama.
```bash
$ ollama pull codellama:7b-instruct
$ bott! config set -k ollama:model -v codellama:7b-instruct
```
3. Configure bott to use Ollama:
```bash
$ bott! config set -k llm openai
```
Secure and ready, bott now utilizes the Ollama model to enhance your terminal experience.
### Commands
#### Queries
Bott excels in aiding you with everyday terminal activities. For instance, when working in a Git repository and wanting to add only the changed JS files to a commit:
```bash
$ bott! query "figure out all the js files that i have changed in the current directory and add them to the commit."
```
Bott keeps track of sessions, allowing you to ask follow-up questions:
```bash
$ bott! query "do the same for html files as well"
```
#### Debug
When troubleshooting commands, bott shines as your debugging assistant. If a command found online, like fetching the OS version, fails:
```bash
$ bott! run cat /etc/os-release
$ bott! debug
```
Bott steps in to investigate and find out why the command is failing on your system.

Enjoy the journey with bott, your trusty companion in the terminal!

