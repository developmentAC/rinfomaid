use clap::{Arg, Command};
use colored::*;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::fs::{File, create_dir_all};
use std::io::{self, Write};

mod toml_extract; // Extract and print the version information according to the toml file

fn show_banner() {
    // banner ref: https://manytools.org/hacker-tools/ascii-banner/

    //logo design: "ticks", use "█" to replace "/\" chars, "_" replaced with space
    let banner = String::from(
        "
\t ██████╗ ██╗███╗   ██╗███████╗ ██████╗ ███╗   ███╗ █████╗ ██╗██████╗ 
\t ██╔══██╗██║████╗  ██║██╔════╝██╔═══██╗████╗ ████║██╔══██╗██║██╔══██╗
\t ██████╔╝██║██╔██╗ ██║█████╗  ██║   ██║██╔████╔██║███████║██║██║  ██║
\t ██╔══██╗██║██║╚██╗██║██╔══╝  ██║   ██║██║╚██╔╝██║██╔══██║██║██║  ██║
\t ██║  ██║██║██║ ╚████║██║     ╚██████╔╝██║ ╚═╝ ██║██║  ██║██║██████╔╝
\t ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═╝      ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚═════╝ 
",
    );

    colour_print(&banner, "purple")
}

#[tokio::main]
async fn main() {
    // Create the output directory if it doesn't exist
    let output_dir = "0_out";
    let _ = create_dir_all(output_dir);

    // show the banner
    show_banner();

    // Display version information from the toml file
    toml_extract::main();

    // print up help message
    let msg = format!("Welcome to Ollama Textual AI Generator!");
    println!("\t {}", msg.bright_yellow().bold());


    // Parse the command-line arguments
    let matches = parse_arguments();

    // Retrieve the prompt input
    let prompt = get_prompt(&matches);

    // Retrieve the model name
    let model = matches.get_one::<String>("model").unwrap().to_string();

    // Retrieve the output file path
    let output_file = matches.get_one::<String>("output").unwrap();

    // update the output file path with the output directory
    let output_file_with_path = format!("{}/{}", output_dir, output_file);
    let msg = format!("Output filepath ").bright_yellow();
    println!(
        "\t {}: {}",
        msg,
        output_file_with_path.bright_green().bold()
    );

    // Connect to Ollama API
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    // Generate response from Ollama AI model
    let res = generate_response(&ollama, &model, &prompt).await;

    // Process the result
    match res {
        // Ok(response) => handle_success(response, output_file, &prompt).await,
        Ok(response) => handle_success(response, &output_file_with_path, &prompt).await,
        Err(_) => handle_failure(),
    }
}

/// Parse the command-line arguments
fn parse_arguments() -> clap::ArgMatches {

    // print up help message
    let msg = format!("\t Sample Command:\n\t cargo run -- --prompt \"What is the capital of France?\" --output \"result.md\" --model \"llama3.2\"\n\n\t Or if you have a file containing the prompt\n\n\t Sample Command:\n\t cargo run --  --prompt-file \"prompt.txt\" --output \"result.md\" --model \"llama3.2\"");

    // Define the command-line interface using clap

    Command::new("Ollama Generator")
        .version("1.0")
        .author("Your Name <you@example.com>")
        // .about("Generates text using Ollama AI models")
        .about(msg)
        .arg_required_else_help(true)
        .arg(
            Arg::new("prompt")
                .short('p')
                .long("prompt")
                .required(false)
                .help("The text prompt to send to the model."),
        )
        .arg(
            Arg::new("prompt_file")
                .short('f')
                .long("prompt-file")
                .required(false)
                .help("File containing the prompt."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(true)
                .help("The file to save the output."),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .required(false)
                .default_value("mistral")
                .help("The model to use for generation."),
        )
        .get_matches()
}

// Retrieve the prompt from either the command line, file, or user input
fn get_prompt(matches: &clap::ArgMatches) -> String {
    if let Some(prompt) = matches.get_one::<String>("prompt") {
        prompt.to_string()
    } else if let Some(prompt_file) = matches.get_one::<String>("prompt_file") {
        std::fs::read_to_string(prompt_file)
            .unwrap_or_else(|_| "\t Failed to read prompt file.".to_string())
    } else {
        // Prompt the user for input if none is provided
        let my_message = format!("\t Enter the prompt : ");
        let my_prompt: String = get_input(&my_message).expect("\t Failed to receive the value...");

        // It may not be necessary to print the prompt again ...
        // println!("\t Prompt set: {}", my_prompt.bright_green().bold());
        // let msg = format!("Prompt set: ").bright_yellow();
        // println!("\t {}: {}", msg, my_prompt.bright_green().bold());

        colour_print("\t Prompt is set", "cyan");
        my_prompt
    }
}

// Asynchronously generate a response from the Ollama AI model
async fn generate_response(ollama: &Ollama, model: &str, prompt: &str) -> Result<String, String> {
    let msg = format!("Prompt ").bright_yellow();
    println!("\t {}: {}", msg, prompt.bright_green().bold());

    // Convert model and prompt to String explicitly
    let model_string = model.to_string();
    let prompt_string = prompt.to_string();

    let res = ollama
        .generate(GenerationRequest::new(model_string, prompt_string))
        .await;

    match res {
        Ok(res) => Ok(res.response),
        Err(_) => Err(String::from("Failed to generate response")),
    }
}

// Handle the success case: save the response to a file
async fn handle_success(response: String, output_file: &str, prompt: &str) {
    colour_print("Response:", "yellow");
    colour_print(&response, "cyan");

    let mut file = File::create(output_file)
        .unwrap_or_else(|_| panic!("Failed to create file {}", output_file));

    writeln!(file, "# Ollama Generation Result\n").unwrap();
    writeln!(file, "## Prompt\n\n{}", prompt).unwrap();
    writeln!(file, "## Response\n\n{}", response).unwrap();

    let msg = format!("Response saved to file: ").bright_yellow().bold();
    println!("\t {}: {}", msg, output_file.bright_green().bold());
}

/// Handle the failure case: print an error message
fn handle_failure() {
    let error_msg = format!("\n\t Oh no, an error...\n\t Failed to generate response.\n\t Is the Ollama server running? \n\t Check that model has been pulled already?").bright_red().bold();

    eprintln!("{}", error_msg);
}

/// Get input from the user
fn get_input(my_message: &str) -> io::Result<String> {
    let mut input = String::new();
    colour_print(my_message, "yellow_noLineFeed");
    io::stdout().flush()?; // Ensure message is printed before reading input
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Print colored text to the console
fn colour_print(text: &str, colour: &str) {
    match colour {
        "flush_green" => {
            print!("\x1b[2K\r"); // Clear the line and move to the beginning
            io::stdout().flush().unwrap();
            print!(" {}", text.bright_green().bold());
            io::stdout().flush().unwrap();
        }
        "green" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_green().bold());
        }
        "green_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_green().bold());
        }
        "red" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_red().bold());
        }
        "cyan" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_cyan().bold());
        }
        "purple" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_purple().bold());
        }
        "purple_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_purple().bold());
        }
        "blue" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_blue().bold());
        }
        "yellow" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_yellow().bold());
        }
        "yellow_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_yellow().bold());
        }
        _ => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_yellow().bold());
        }
    }
}

// run command: cargo run -- -p "Why is the sky blue?" -m mistral -o output.md
// cargo run -- --prompt "What is the capital of France?" --output "result.md" --model "mistral"
