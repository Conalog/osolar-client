use osolar_client::OsolarLinkClient;

fn main() {
    let api_key = std::env::var("OSOLAR_API_KEY").unwrap_or_else(|_| {
        eprintln!("OSOLAR_API_KEY is required");
        std::process::exit(1);
    });

    let client = OsolarLinkClient::new(api_key);

    match client.list_linked_plants() {
        Ok(response) => {
            let count = response.data.as_ref().map(std::vec::Vec::len).unwrap_or(0);
            println!(
                "{{\"success\":{},\"linkedPlantCount\":{}}}",
                response.success, count
            );
        }
        Err(err) => {
            eprintln!("Live smoke test failed: {err}");
            std::process::exit(1);
        }
    }
}
