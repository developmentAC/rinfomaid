# Rinfomaid:  A Textual AI Application written in Rust

![logo](graphics/rinfomaid_logo.png)

Date: 15 March 2025

[Oliver Bonham-Carter](https://www.oliverbonhamcarter.com/)

Email: obonhamcarter at allegheny.edu

[![MIT Licence](https://img.shields.io/bower/l/bootstrap)](https://opensource.org/licenses/MIT)


The **RinfoMaid** is a Rust command-line tool that interacts with the Ollama API to generate text using AI models. The program can either take a prompt from the command line, a prompt file, or ask the user to enter a prompt. It then sends the prompt to the Ollama API and retrieves a generated response, which is saved to a specified output file in markdown format. This project is a learning tool for programming in Rust. Cool, right?

## Table of Contents

- [Rinfomaid:  A Textual AI Application written in Rust](#rinfomaid--a-textual-ai-application-written-in-rust)
  - [Table of Contents](#table-of-contents)
  - [Dependencies](#dependencies)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Run with Prompt](#run-with-prompt)
    - [Run with Prompt File](#run-with-prompt-file)
    - [Run with Model](#run-with-model)
  - [Explanation of Code](#explanation-of-code)
    - [Main Flow](#main-flow)
    - [Functions](#functions)
  - [Concluding Thoughts](#concluding-thoughts)
  - [Special Notes](#special-notes)
    - [Definitions of Terms](#definitions-of-terms)
    - [Ethical Note](#ethical-note)
    - [A Work In Progress](#a-work-in-progress)

## Dependencies
This `README.md` file should provide sufficient context for a user to understand, set up, and use the program.

To run this _RinfoMaid_, you will need to add a few dependencies in your `Cargo.toml` file. These dependencies allow the program to parse command-line arguments, interact with the Ollama API, and handle colored output.

Here are the necessary dependencies:

- **`clap`**: For parsing command-line arguments.
- **`colored`**: For printing colored text to the terminal.
- **`ollama_rs`**: To interact with the Ollama API and generate text.
- **`tokio`**: To support asynchronous execution using `async/await`.

Add the following dependencies to your `Cargo.toml` file under `[dependencies]`:

```toml
[dependencies]
clap = "4.1.0"
colored = "2.0"
ollama_rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Installation

1. **Clone the Repository**:

   If you have not already cloned the repository, use the following command:

   ```bash
   git clone https://github.com/yourusername/ollama-generator.git
   cd ollama-generator
   ```

2. **Add Dependencies**:

   As mentioned in the dependencies section, make sure to include the necessary libraries in your `Cargo.toml` file. You can also use `cargo add` to add the dependencies directly:

   ```bash
   cargo add clap colored ollama_rs tokio
   ```

3. ### Set Up Ollama and its Local Models

   Ollama may be downloaded from [https://ollama.com/](https://ollama.com/). The below commands will install the models that Ollama will require to perform its functions. Note, a typical model for Ollama is about 4 GB in size. As there are manty diofferent models that you can experiment with at [https://ollama.com/search](https://ollama.com/search).

   ``` bash
   ollama pull mistral
   ```

4. **Build the Project**:

   To build the project, run the following:

   ```bash
   cargo build
   ```

5. **Run the Program**:

   You can run the program using the following command:

   ```bash
   cargo run -- --prompt "What is the capital of France?" --output "result.md" --model "mistral"
   ```
_Note: the explainations for these options are provided below._

## Usage

The program supports the following command-line arguments:

- **`-p` or `--prompt`**: A direct prompt you want to send to the model (optional).
- **`-f` or `--prompt-file`**: A file that contains the prompt (optional).
- **`-o` or `--output`**: The output file to save the generated response (required).
- **`-m` or `--model`**: The AI model to use for text generation (required). Defaults to "mistral".

### Run with Prompt

If you want to provide a prompt directly from the command line, use the `-p` argument:

```bash
cargo run -- -p "What is the capital of Pennsylvania?" -m mistral -o output.md
```

This will generate text based on the provided prompt and save the result in markdown formatting to a created output directory, `0_out/output.md`.

### Run with Prompt File

You can also provide a prompt from a file using the `-f` argument:

```bash
cargo run -- --prompt-file "prompt.txt" --output "result.md" --model "mistral"
```

This will read the prompt from the file `prompt.txt`, generate a response, and save it to `result.md`.

### Run with Model

You can specify the model to use for generation with the `-m` argument. If not specified, it defaults to `mistral`:

```bash
cargo run -- --prompt "What is the capital of France?" --output "result.md" --model "mistral"
```

## Explanation of Code

### Main Flow

1. **Argument Parsing**:
   - The program starts by using `clap` to parse the command-line arguments. It looks for the following arguments:
     - `prompt`: Direct input for the prompt.
     - `prompt_file`: A file containing the prompt.
     - `output`: The file where the result will be saved.
     - `model`: The AI model to use for text generation.

2. **Input Prompt**:
   - The program checks if the user has provided a prompt via the `-p` or `-f` argument. If neither is provided, the program prompts the user to enter a prompt interactively.

3. **Generate Response**:
   - The program connects to the Ollama API (`http://localhost:11434`) and sends the prompt to the selected model. It waits for the response asynchronously.

4. **Handling Results**:
   - If the request is successful, the response is printed in cyan, and the result is saved to the specified output file in markdown format.
   - If the request fails (e.g., the server is not running), an error message is displayed.

### Functions

- **`parse_arguments()`**: This function sets up the command-line argument parsing using `clap`. It defines the expected arguments and their descriptions.
- **`get_prompt()`**: This function retrieves the prompt. It checks if a prompt is provided as a command-line argument or a file. If not, it asks the user for input.
- **`generate_response()`**: This asynchronous function sends the prompt to the Ollama API and retrieves the response.
- **`handle_success()`**: This function handles the success case. It prints the response in cyan and saves it to the output file.
- **`handle_failure()`**: This function prints an error message if the request to Ollama fails.
- **`get_input()`**: This function prompts the user for input when no prompt is provided.
- **`colour_print()`**: A utility function that prints text in different colors using the `colored` crate.

## Concluding Thoughts

This project demonstrates how to interact with an AI model to generate text using the Ollama API. The tool is flexible, allowing users to provide prompts in different ways (via command line, file, or interactive input). The results are saved in a markdown format, making it easy to view and share the generated responses.

If you have any issues or suggestions, feel free to open an issue in the repository or submit a pull request.

---

## Special Notes

**When uising Ollama**: Ensure that the Ollama server is running on `http://localhost:11434` for the program to work.
 
### Definitions of Terms

- **Dependencies**: This section outlines the necessary dependencies for the project and shows how to add them to the `Cargo.toml` file.
- **Installation**: Describes how to clone the project, install the dependencies, and build the project.
- **Usage**: Shows examples of how to run the program with different configurations (direct prompt, file prompt, specifying model).
- **Explanation of Code**: Provides an overview of the main flow and the functions in the code to help users understand how the program works.

### Ethical Note

While there is a lot of convenience in using AI to prepare drafts of letters and other communications, in all this automation, it is important to have a human presence to preside over the generated textual (or graphical work). While AI systems excel at processing vast amounts of data and executing tasks with remarkable efficiency, they lack the nuanced understanding and ethical judgment inherent to human cognition, in addition to the sense of ethics that ought to come from the human world.

Involving ethics in decisions where machines have made the choices (as strange as that may seem) is essential in domains involving communication. Human oversight ensures that communications, whether they involve customer interactions, inter-office correspondence, or public statements, adhere to ethical standards, tone, and context sensitivity. In addition, decisions influenced by AI algorithms must be subjected to human judgment before implementation. Human evaluators can consider broader implications, ethical ramifications, and potential biases that AI systems might overlook. This "human-touch" can therefore help to safeguard against the potential and unintended consequences which may occur at the intersection of data and decision-making, to name one such area.

With this in mind, the _RInfomaid_ project must be used responsibly. The project is to serve educational purposes -- it is to instruct on the uses of AI, allow for discovery and to entertain (in a way!). Please use _RInfomaid_ responsibly.

---

### A Work In Progress

Check back often to see the evolution of the project!! _RInfomaid_ is a work-in-progress. Updates will come periodically.

If you would like to contribute to this project, __then please do!__ For instance, if you see some low-hanging fruit or task that you could easily complete, that could add value to the project, then I would love to have your insight.

Otherwise, please create an Issue for bugs or errors. Since I am a teaching faculty member at Allegheny College, I may not have all the time necessary to quickly fix the bugs. I welcome the OpenSource Community to further the development of this project. Much thanks in advance. 

If you appreciate this project, please consider clicking the project's _Star_ button. :-)
