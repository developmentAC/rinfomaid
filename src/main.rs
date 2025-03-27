use clap::{Arg, Command};
use colored::*;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::fs::{File, create_dir_all};
use std::io::{self, Write};

mod toml_extract; // Extract and print the version information according to the toml file

// Function to display the banner
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

    // Print the banner in purple color
    colour_print(&banner, "purple")
}

#[tokio::main]
async fn main() {
    // Create the output directory if it doesn't exist
    let output_dir = "0_out";
    let _ = create_dir_all(output_dir);

    // Show the banner
    show_banner();

    // Display version information from the toml file
    toml_extract::main();

    // Print a welcome message and quick help instruction
    let msg = format!(
        "Welcome to Ollama Textual AI Generator!\n\t Use: \"cargo run -- --bighelp\" for quick help."
    );
    println!("\t {}", msg.bright_yellow().bold());
    println!(
        "\t {}",
        "-------------------------------------------------\n"
            .bright_yellow()
            .bold()
    );

    // Parse the command-line arguments
    let matches = parse_arguments();

    // Retrieve the Big Help message flag
    let big_help = matches.get_flag("bighelp");

    // If big_help is requested, display the help message and exit
    if big_help {
        get_big_help();
        return;
    }

    // Retrieve the prompt input
    let prompt = get_prompt(&matches);

    // Retrieve the model name
    let model = matches.get_one::<String>("model").unwrap().to_string();

    // Retrieve the output file path
    let output_file = matches.get_one::<String>("output").unwrap();

    // Retrieve the number of results
    let num_results: usize = matches
        .get_one::<String>("num_results")
        .unwrap()
        .parse()
        .expect("Invalid number of results");

    // Update the output file path with the output directory
    let output_file_with_path = format!("{}/{}", output_dir, output_file);
    let msg = format!("Output filepath ").bright_yellow().bold();
    println!(
        "\t {}: {}",
        msg,
        output_file_with_path.bright_green().bold()
    );

    // Connect to Ollama API
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    // Generate response from Ollama AI model
    let res = generate_response(&ollama, &model, &prompt, num_results).await;

    // Process the result
    match res {
        Ok(response) => handle_success(response, &output_file_with_path, &prompt, &model).await,
        Err(_) => handle_failure(),
    }
}

// Print out the help message
fn get_big_help() {
    let msg = format!("\n\t cargo run -- --prompt \"What is the capital of France?\"  --output \"result.md\"  --model \"llama3.2\"  --num-results 2").bright_cyan().bold();
    println!("{}", msg);
}

/// Parse the command-line arguments
fn parse_arguments() -> clap::ArgMatches {
    Command::new("Ollama Generator")
        .version("1.0")
        .author("Oliver Bonham-Carter <obonhamcarter@allegheny.edu>")
        .about("Generates text using Ollama AI models")
        .arg_required_else_help(true)
        .arg(
            Arg::new("bighelp")
                .short('b')
                .long("bighelp")
                .action(clap::ArgAction::SetTrue)
                .help("Get a sample prompt to send to the model."),
        )
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
                .required(false)
                .default_value("output")
                .help("The file to save the output."),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .required(false)
                // Change the default model to "llama3.2" or whatever model you want
                .default_value("llama3.2")
                .help("The model to use for generation."),
        )
        .arg(
            Arg::new("num_results")
                .short('n')
                .long("num-results")
                .required(false)
                .default_value("1")
                .help("The number of results to generate."),
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

        // Print a message indicating that the prompt is set
        colour_print("\t Prompt is set", "cyan");
        my_prompt
    }
}

// Asynchronously generate a response from the Ollama AI model
async fn generate_response(
    ollama: &Ollama,
    model: &str,
    prompt: &str,
    num_results: usize,
) -> Result<Vec<String>, String> {
    let msg = format!("Prompt ").bright_yellow().bold();
    println!("\t {}: {}", msg, prompt.bright_green().bold());

    // Convert model and prompt to String explicitly
    let model_string = model.to_string();
    let prompt_string = prompt.to_string();

    let mut results = Vec::new();
    for _ in 0..num_results {
        let res = ollama
            .generate(GenerationRequest::new(
                model_string.clone(),
                prompt_string.clone(),
            ))
            .await;

        match res {
            Ok(res) => results.push(res.response),
            Err(_) => return Err(String::from("Failed to generate response")),
        }
    }

    Ok(results)
}

// Handle the success case: save the response to a file
async fn handle_success(responses: Vec<String>, output_file: &str, prompt: &str, model: &str) {
    // Print the responses in yellow color
    colour_print("\t Responses:", "yellow");

    // Create the output file
    let mut file = File::create(output_file)
        .unwrap_or_else(|_| panic!("Failed to create file {}", output_file));

    // Write the results to the file
    writeln!(file, "# Ollama Generation Result\n").unwrap();
    writeln!(file, "## Model: {}\n", model).unwrap();
    writeln!(file, "## Prompt\n\n{}", prompt).unwrap();

    // Write each response to the file
    for (i, response) in responses.iter().enumerate() {
        let msg = format!("\t   {}", response);
        colour_print(&msg, "cyan");
        writeln!(file, "\n## Response {}\n{}", i + 1, response).unwrap();
    }

    // Print a blank line at the end of the markdown file
    writeln!(file, "").unwrap(); 

    // Print a message indicating that the responses are saved to the file
    let msg = format!("Responses saved to file: ").bright_yellow().bold();
    println!("\t {}: {}", msg, output_file.bright_green().bold());
}

// Handle the failure case: print an error message
fn handle_failure() {
    let error_msg = format!("\n\t Oh no, an error...\n\t Failed to generate response.\n\t Is the Ollama server running? \n\t Check that model has been pulled already?").bright_red().bold();

    eprintln!("{}", error_msg);
}

// Get input from the user
fn get_input(my_message: &str) -> io::Result<String> {
    let mut input = String::new();
    colour_print(my_message, "yellow_noLineFeed");
    io::stdout().flush()?; // Ensure message is printed before reading input
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// Print colored text to the console
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

// Sample run Commands:
// cargo run -- -p "Why is the sky blue?" -m mistral -o output.md --num_results 2
// cargo run -- --prompt "What is the capital of France?" --output "result.md" --model "llama3.2" --num-results 2
