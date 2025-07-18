// Import necessary libraries for CLI, colored output, Ollama API, and file operations

// Apologies for the messy code! One day, I will clean it up and make it more readable! In the meantime, the comments should help clarify the purpose of each section.

use clap::{Arg, Command};
use colored::*;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::fs::{File, create_dir_all};
use std::io::{self, Write};
use std::path::Path;

// Import custom modules for version extraction and RAG functionality
mod rag;
mod toml_extract; // Extract and print the version information according to the toml file // RAG system for local document processing

// Function to display the ASCII art banner at program startup
fn show_banner() {
    // ASCII art banner reference: https://manytools.org/hacker-tools/ascii-banner/
    // Logo design uses "ticks" style, with "█" replacing "/\" characters, "_" replaced with spaces
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

    // Print the banner in purple color using our custom color printing function
    colour_print(&banner, "purple")
}

// Main asynchronous function - entry point of the application
#[tokio::main]
async fn main() {
    // Display the ASCII art banner
    show_banner();

    // Display version information extracted from Cargo.toml
    toml_extract::main();

    // Print a welcome message with instructions for getting help
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

    // Parse command-line arguments using clap
    let matches = parse_arguments();

    // Handle RAG-specific commands (build, remove, status) if provided
    if let Some(rag_command) = matches.get_one::<String>("rag") {
        handle_rag_command(rag_command).await;
        return; // Exit early after handling RAG command
    }

    // Check if user requested the big help message
    let big_help = matches.get_flag("bighelp");

    // Display comprehensive help and exit if requested
    if big_help {
        get_big_help();
        return;
    }

    // Check if user wants to query the local RAG knowledge base
    let use_local = matches.get_flag("use_local");

    // Retrieve the user's prompt from various sources (CLI, file, or interactive input)
    let prompt = get_prompt(&matches);

// print the prompt that will be sent to the model
    let msg = format!("Prompt ").bright_yellow().bold();
    println!("\t {}: {}\n", msg, prompt.bright_green().bold());

    // If using local RAG, try to answer with local model first
    if use_local {
        if let Ok(response) = handle_local_rag_query(&prompt).await {
            // Extract output parameters for file saving
            let output_file = matches.get_one::<String>("output").unwrap();
            let output_dir = "0_out"; // Standard output directory
            let model = "llama3.2 (local RAG)"; // Indicate it's using local RAG

            // Generate unique output file path
            let output_file_with_path = generate_unique_output_path(output_dir, output_file);
            let msg = format!("Output filepath ").bright_yellow().bold();
            println!(
                "\t {}: {}",
                msg,
                output_file_with_path.bright_green().bold()
            );

            // Save response to file and display
            println!("{}", response);
            handle_success_local_rag(vec![response], &output_file_with_path, &prompt, model).await;
            return; // Exit early if local RAG successfully answered
        }
        // If local RAG fails, continue to standard Ollama processing
    }

    // Extract standard Ollama generation parameters from command-line arguments
    let model = matches.get_one::<String>("model").unwrap().to_string();
    let output_file = matches.get_one::<String>("output").unwrap();
    let output_dir = "0_out"; // Standard output directory

    // Parse the number of results to generate
    let num_results: usize = matches
        .get_one::<String>("num_results")
        .unwrap()
        .parse()
        .expect("Invalid number of results");

    // Generate a unique output file path to avoid overwriting existing files
    let output_file_with_path = generate_unique_output_path(output_dir, output_file);
    let msg = format!("Output filepath ").bright_yellow().bold();
    println!(
        "\t {}: {}",
        msg,
        output_file_with_path.bright_green().bold()
    );

    // Initialize connection to Ollama API server
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    // Generate response(s) from the Ollama AI model
    let res = generate_response(&ollama, &model, &prompt, num_results).await;

    // Process the result and either save successful responses or display error
    match res {
        Ok(response) => handle_success(response, &output_file_with_path, &prompt, &model).await,
        Err(_) => handle_failure(),
    }
}

// Display comprehensive help message with examples for all major features
fn get_big_help() {
    // Standard AI generation examples
    let msg = format!("\n\t 🤖 Standard AI Generation:")
        .bright_cyan()
        .bold();
    println!("{}", msg);
    let msg = format!("\t cargo run -- --prompt \"What is the capital of France?\"  --output \"result.md\"  --model \"llama3.2\"  --num-results 2").bright_cyan().bold();
    println!("{}", msg);

    // RAG (Retrieval-Augmented Generation) command examples
    let msg = format!("\n\t 📚 RAG (Retrieval-Augmented Generation) Commands:")
        .bright_cyan()
        .bold();
    println!("{}", msg);
    let msg = format!(
        "\t cargo run -- --rag build                    # Build local model from data/ directory"
    )
    .bright_cyan()
    .bold();
    println!("{}", msg);
    let msg = format!("\t cargo run -- --rag status                   # Check local model status")
        .bright_cyan()
        .bold();
    println!("{}", msg);
    let msg = format!("\t cargo run -- --rag remove                   # Remove local model")
        .bright_cyan()
        .bold();
    println!("{}", msg);

    // Local knowledge base querying examples
    let msg = format!("\n\t 🔍 Using Local Knowledge Base:")
        .bright_cyan()
        .bold();
    println!("{}", msg);
    let msg = format!("\t cargo run -- --use-local --prompt \"Tell me about AstroBill\"")
        .bright_cyan()
        .bold();
    println!("{}", msg);

    // Supported file types information
    let msg = format!("\n\t 📁 Supported file types in data/ directory: PDF, TXT, MD")
        .bright_yellow()
        .bold();
    println!("{}", msg);
}

/// Parse and configure command-line arguments using clap
/// Returns: ArgMatches containing parsed command-line arguments
fn parse_arguments() -> clap::ArgMatches {
    Command::new("Ollama Generator")
        .version("1.0")
        .author("Oliver Bonham-Carter <obonhamcarter@allegheny.edu>")
        .about("Generates text using Ollama AI models with optional RAG support")
        .arg_required_else_help(true) // Show help if no arguments provided
        .arg(
            Arg::new("bighelp")
                .short('b')
                .long("bighelp")
                .action(clap::ArgAction::SetTrue)
                .help("Display comprehensive help with examples."),
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
                .help("File containing the prompt to send to the model."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(false)
                .default_value("output.md")
                .help("The output file to save the generated response."),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .required(false)
                .default_value("llama3.2") // Default model for generation
                .help("The Ollama model to use for text generation."),
        )
        .arg(
            Arg::new("num_results")
                .short('n')
                .long("num-results")
                .required(false)
                .default_value("1")
                .help("The number of results to generate (1-10)."),
        )
        .arg(
            Arg::new("rag")
                .short('r')
                .long("rag")
                .required(false)
                .help("RAG system command: 'build', 'remove', or 'status'."),
        )
        .arg(
            Arg::new("use_local")
                .short('l')
                .long("use-local")
                .action(clap::ArgAction::SetTrue)
                .help("Use local RAG knowledge base for query processing."),
        )
        .get_matches()
}

// Retrieve the prompt from command-line argument, file, or interactive user input
// Priority: 1) CLI prompt argument, 2) prompt file, 3) interactive user input
fn get_prompt(matches: &clap::ArgMatches) -> String {
    // Check if prompt was provided directly via command line
    if let Some(prompt) = matches.get_one::<String>("prompt") {
        prompt.to_string()
    }
    // Check if prompt was provided via file
    else if let Some(prompt_file) = matches.get_one::<String>("prompt_file") {
        // Attempt to read the prompt from the specified file
        std::fs::read_to_string(prompt_file)
            .unwrap_or_else(|_| "\t Failed to read prompt file.".to_string())
    }
    // If no prompt source specified, prompt user for interactive input
    else {
        let my_message = format!("\t Enter the prompt : ");
        let my_prompt: String = get_input(&my_message).expect("\t Failed to receive the value...");

        // Confirm to user that prompt has been set
        colour_print("\t Prompt is set", "cyan");
        my_prompt
    }
}

// Asynchronously generate response(s) from the Ollama AI model
// Parameters:
//   - ollama: Reference to Ollama client instance
//   - model: Name of the model to use for generation
//   - prompt: The text prompt to send to the model
//   - num_results: Number of separate responses to generate
// Returns: Result containing vector of generated responses or error string
async fn generate_response(
    ollama: &Ollama,
    model: &str,
    prompt: &str,
    num_results: usize,
) -> Result<Vec<String>, String> {
    // Display the prompt that will be sent to the model
    let msg = format!("Prompt ").bright_yellow().bold();
    println!("\t {}: {}", msg, prompt.bright_green().bold());

    // Convert parameters to owned strings for use in async operations
    let model_string = model.to_string();
    let prompt_string = prompt.to_string();

    // Generate the requested number of responses
    let mut results = Vec::new();
    for _ in 0..num_results {
        // Send generation request to Ollama API
        let res = ollama
            .generate(GenerationRequest::new(
                model_string.clone(),
                prompt_string.clone(),
            ))
            .await;

        // Handle the response from Ollama
        match res {
            Ok(res) => results.push(res.response),
            Err(_) => return Err(String::from("Failed to generate response")),
        }
    }

    Ok(results)
}

// Handle successful response generation by displaying and saving results
// Parameters:
//   - responses: Vector of generated text responses
//   - output_file: Path where the results should be saved
//   - prompt: Original prompt that was sent to the model
//   - model: Name of the model that generated the responses
async fn handle_success(responses: Vec<String>, output_file: &str, prompt: &str, model: &str) {
    // Display header for the responses
    colour_print("\t Responses:", "yellow");

    // Create the output file at the specified path
    let mut file = File::create(output_file)
        .unwrap_or_else(|_| panic!("Failed to create file {}", output_file));

    // Write markdown-formatted header information to the file
    writeln!(file, "# Ollama Generation Result\n").unwrap();
    writeln!(file, "## Model: {}\n", model).unwrap();
    writeln!(file, "## Prompt\n\n{}", prompt).unwrap();

    // Display each response to the terminal and write to file
    for (i, response) in responses.iter().enumerate() {
        // Display response in cyan color to terminal
        let msg = format!("\t   {}", response);
        colour_print(&msg, "cyan");

        // Write response to file with numbered sections
        writeln!(file, "\n## Response {}\n{}", i + 1, response).unwrap();
    }

    // Add a blank line at the end of the markdown file for proper formatting
    writeln!(file, "").unwrap();

    // Confirm successful file save to user
    let msg = format!("Responses saved to file: ").bright_yellow().bold();
    println!("\t {}: {}", msg, output_file.bright_green().bold());
}

// Handle successful local RAG response generation by saving results to file
// Parameters:
//   - responses: Vector of generated text responses from local RAG
//   - output_file: Path where the results should be saved
//   - prompt: Original prompt that was sent to the model
//   - model: Name of the model that generated the responses
async fn handle_success_local_rag(
    responses: Vec<String>,
    output_file: &str,
    prompt: &str,
    model: &str,
) {
    // Create the output file at the specified path
    let mut file = File::create(output_file)
        .unwrap_or_else(|_| panic!("Failed to create file {}", output_file));

    // Write markdown-formatted header information to the file
    writeln!(file, "# Local RAG Generation Result\n").unwrap();
    writeln!(file, "## Model: {}\n", model).unwrap();
    writeln!(file, "## Prompt\n\n{}", prompt).unwrap();

    // Write each response to file with numbered sections
    for (i, response) in responses.iter().enumerate() {
        // Write response to file with numbered sections
        writeln!(file, "\n## Response {}\n{}", i + 1, response).unwrap();
    }

    // Add a blank line at the end of the markdown file for proper formatting
    writeln!(file, "").unwrap();

    // Confirm successful file save to user
    let msg = format!("Local RAG response saved to file: ")
        .bright_yellow()
        .bold();
    println!("\t {}: {}", msg, output_file.bright_green().bold());
}

// Handle failure case when response generation fails
// Displays user-friendly error message with troubleshooting suggestions
fn handle_failure() {
    let error_msg = format!(
        "\n\t Oh no, an error...\n\t Failed to generate response.\n\t Is the Ollama server running? \n\t Check that model has been pulled already?"
    ).bright_red().bold();

    eprintln!("{}", error_msg);
}

// Get input from the user via interactive terminal prompt
// Parameters:
//   - my_message: The prompt message to display to the user
// Returns: Result containing the user's input string or IO error
fn get_input(my_message: &str) -> io::Result<String> {
    let mut input = String::new();

    // Display the prompt message without a newline
    colour_print(my_message, "yellow_noLineFeed");
    io::stdout().flush()?; // Ensure message is displayed before reading input

    // Read user input from stdin
    io::stdin().read_line(&mut input)?;

    // Return trimmed input (removes trailing newline)
    Ok(input.trim().to_string())
}

// Print colored text to the console with various formatting options
// Parameters:
//   - text: The text to print
//   - colour: Color/formatting specification string
// Supported colors: flush_green, green, green_noLineFeed, red, cyan, purple,
//                   purple_noLineFeed, blue, yellow, yellow_noLineFeed
fn colour_print(text: &str, colour: &str) {
    match colour {
        // Clear line and print green text with flush (for progress indicators)
        "flush_green" => {
            print!("\x1b[2K\r"); // Clear the line and move to the beginning
            io::stdout().flush().unwrap();
            print!(" {}", text.bright_green().bold());
            io::stdout().flush().unwrap();
        }
        // Clear line and print green text with newline
        "green" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_green().bold());
        }
        // Clear line and print green text without newline
        "green_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_green().bold());
        }
        // Clear line and print red text with newline
        "red" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_red().bold());
        }
        // Clear line and print cyan text with newline
        "cyan" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_cyan().bold());
        }
        // Clear line and print purple text with newline
        "purple" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_purple().bold());
        }
        // Clear line and print purple text without newline
        "purple_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_purple().bold());
        }
        // Clear line and print blue text with newline
        "blue" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_blue().bold());
        }
        // Clear line and print yellow text with newline
        "yellow" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_yellow().bold());
        }
        // Clear line and print yellow text without newline
        "yellow_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_yellow().bold());
        }
        // Default case: print yellow text with newline
        _ => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_yellow().bold());
        }
    }
}

// ==================== RAG SYSTEM HANDLER FUNCTIONS ====================

// Handle RAG-specific commands (build, remove, status)
// Parameters:
//   - command: The RAG command to execute ("build", "remove", or "status")
async fn handle_rag_command(command: &str) {
    // Initialize RAG system with agentic directory for model storage and data directory for source files
    let mut rag_system = rag::RagSystem::new("agentic", "data");

    match command {
        // Build local knowledge base from documents in data/ directory
        "build" => {
            if let Err(e) = rag_system.build_local_model() {
                colour_print(&format!("\t Error building local model: {}", e), "red");
            }
        }
        // Remove local knowledge base and all associated files
        "remove" => {
            if let Err(e) = rag_system.remove_local_model() {
                colour_print(&format!("\t Error removing local model: {}", e), "red");
            }
        }
        // Display status information about the local knowledge base
        "status" => {
            if rag_system.is_model_available() {
                // Attempt to load the model and display statistics
                if let Ok(true) = rag_system.load_model() {
                    let (doc_count, chunk_count) = rag_system.get_stats();
                    colour_print(
                        &format!(
                            "\t Local model is available with {} documents and {} chunks",
                            doc_count, chunk_count
                        ),
                        "green",
                    );
                } else {
                    colour_print("\t Local model exists but failed to load", "red");
                }
            } else {
                colour_print(
                    "\t No local model available. Use 'cargo run -- --rag build' to create one.",
                    "yellow",
                );
            }
        }
        // Handle invalid commands
        _ => {
            colour_print(
                "\t Invalid RAG command. Use 'build', 'remove', or 'status'.",
                "red",
            );
        }
    }
}

// Handle local RAG queries by searching the knowledge base and generating responses
// Parameters:
//   - query: The user's question/query to search for in the local knowledge base
// Returns: Result containing formatted response or error message
async fn handle_local_rag_query(query: &str) -> Result<String, String> {
    // Initialize RAG system with standard directories
    let mut rag_system = rag::RagSystem::new("agentic", "data");

    // Check if a local model exists
    if !rag_system.is_model_available() {
        return Err(
            "No local model available. Use 'cargo run -- --rag build' to create one.".to_string(),
        );
    }

    // Attempt to load the local model
    if let Err(e) = rag_system.load_model() {
        return Err(format!("Failed to load local model: {}", e));
    }

    // Inform user that local search is being performed
    colour_print("\t Searching local knowledge base...", "cyan");

    // Search for relevant documents using TF-IDF scoring (top 3 results)
    let search_results = rag_system.search_local(query, 3);

    // Check if any relevant documents were found
    if search_results.is_empty() {
        colour_print("\t No relevant information found in local model.", "yellow");
        colour_print(
            "\t The request is outside the local model's capabilities.",
            "yellow",
        );
        colour_print("\t Falling back to Ollama model...", "cyan");
        return Err("No relevant local information found".to_string());
    }

    // Check if the best match meets minimum relevance threshold
    let best_score = search_results
        .first()
        .map(|(score, _)| *score)
        .unwrap_or(0.0);
    if best_score < 0.001 {
        // Minimum threshold for relevance
        colour_print(
            "\t The request is outside the local model's capabilities.",
            "yellow",
        );
        colour_print("\t Falling back to Ollama model...", "cyan");
        return Err("Query outside local model capabilities".to_string());
    }

    // Generate context from the most relevant search results
    let context = generate_context_from_search_results(&search_results, &rag_system);

    // Create enhanced prompt that includes local context
    let enhanced_prompt = format!(
        "Based on the following context from local documents, please answer the question:\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer:",
        context, query
    );

    // Use Ollama to generate a response using the enhanced prompt with local context
    let ollama = Ollama::new("http://localhost".to_string(), 11434);
    let model = "llama3.2".to_string();

    match ollama
        .generate(GenerationRequest::new(model, enhanced_prompt))
        .await
    {
        Ok(response) => {
            // Format successful response with source attribution
            colour_print("\t Response generated using local knowledge base:", "green");
            let formatted_response = format!(
                "\n\t📚 **Local Knowledge Base Response:**\n\t{}\n\n\t**Sources used:**\n{}",
                response.response.bright_cyan(),
                format_sources(&search_results, &rag_system)
            );
            Ok(formatted_response)
        }
        Err(e) => Err(format!("Failed to generate response: {}", e)),
    }
}

// Generate context string from search results for use in enhanced prompts
// Parameters:
//   - search_results: Vector of (score, chunk) tuples from the search
//   - rag_system: Reference to the RAG system for document lookup
// Returns: Formatted context string containing relevant document excerpts
fn generate_context_from_search_results(
    search_results: &[(f32, &rag::DocumentChunk)],
    rag_system: &rag::RagSystem,
) -> String {
    let mut context = String::new();

    // Process up to 3 most relevant search results
    for (score, chunk) in search_results.iter().take(3) {
        // Get the document title for each chunk
        if let Some(doc) = rag_system.get_document_by_id(&chunk.document_id) {
            context.push_str(&format!(
                "Document: {}\nRelevance Score: {:.4}\nContent: {}\n\n",
                doc.title, score, chunk.content
            ));
        }
    }

    context
}

// Format source attribution for display to user
// Parameters:
//   - search_results: Vector of (score, chunk) tuples from the search
//   - rag_system: Reference to the RAG system for document lookup
// Returns: Formatted string showing sources and their relevance scores
fn format_sources(
    search_results: &[(f32, &rag::DocumentChunk)],
    rag_system: &rag::RagSystem,
) -> String {
    let mut sources = String::new();

    // Format up to 3 sources with numbering and relevance scores
    for (i, (score, chunk)) in search_results.iter().take(3).enumerate() {
        if let Some(doc) = rag_system.get_document_by_id(&chunk.document_id) {
            sources.push_str(&format!(
                "\t  {}. {} (Relevance: {:.4})\n",
                i + 1,
                doc.title.bright_blue(),
                score
            ));
        }
    }

    sources
}

// ==================== FILE HANDLING FUNCTIONS ====================

// Generate a unique output file path to prevent overwriting existing files
// Parameters:
//   - output_dir: Directory where the output file should be created
//   - output_file: Desired filename for the output
// Returns: Unique file path with number suffix if necessary (e.g., "file_1.md")
fn generate_unique_output_path(output_dir: &str, output_file: &str) -> String {
    // Ensure the output directory exists, create if necessary
    if let Err(e) = create_dir_all(output_dir) {
        eprintln!("Failed to create output directory {}: {}", output_dir, e);
        std::process::exit(1);
    }

    // Construct the base file path
    let base_path = format!("{}/{}", output_dir, output_file);

    // If the file doesn't exist, return the original path
    if !Path::new(&base_path).exists() {
        return base_path;
    }

    // Extract the filename stem (without extension) and extension
    let path = Path::new(output_file);
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("output");
    let extension = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("md");

    // Generate numbered variations until we find one that doesn't exist
    let mut counter = 1;
    loop {
        // Format new filename with counter
        let new_filename = if extension.is_empty() {
            format!("{}_{}", stem, counter)
        } else {
            format!("{}_{}.{}", stem, counter, extension)
        };

        let new_path = format!("{}/{}", output_dir, new_filename);

        // Check if this numbered variation is available
        if !Path::new(&new_path).exists() {
            return new_path;
        }

        counter += 1;

        // Safety check to prevent infinite loop (though very unlikely)
        if counter > 9999 {
            eprintln!(
                "Error: Too many output files with similar names. Please clean up the output directory."
            );
            std::process::exit(1);
        }
    }
}

// ==================== SAMPLE USAGE COMMANDS ====================
// Sample run Commands:
// cargo run -- -p "Why is the sky blue?" -m mistral -o output.md --num-results 2
// cargo run -- --prompt "What is the capital of France?" --output "result.md" --model "llama3.2" --num-results 2
