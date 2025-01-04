# Loadster

Loadster is a simple load testing tool that allows you to test the performance of your web applications by sending concurrent HTTP requests.

## Features

- Supports multiple HTTP methods: GET, POST, PUT, DELETE, PATCH
- Customizable number of concurrent users
- Configurable request timeout
- Ability to add custom headers and body
- Verbose output for detailed request information
- Save results to a file
- Includes timestamp for each request

## Usage

To use Loadster, run the following command:

```sh
cargo run -- --url <URL> [OPTIONS]
```

### Options

- `-u, --url <URL>`: The target URL for the load test.
- `-m, --method <METHOD>`: The HTTP method to use (default: GET). Supported methods: GET, POST, PUT, DELETE, PATCH.
- `-c, --users <USERS>`: The number of concurrent users (default: 10).
- `-t, --timeout <TIMEOUT>`: The timeout for each request in seconds (default: 30).
- `-H, --headers <HEADERS>`: Additional headers to include in the requests.
- `-b, --body <BODY>`: The body of the request (for POST, PUT, PATCH methods).
- `-v, --verbose`: Enable verbose output.
- `-o, --output <OUTPUT>`: Save the results to a file.

### Examples

Basic GET request:
```sh
cargo run -- --url <URL>
```

POST request with headers and body:
```sh
cargo run -- --url <URL> --method post --headers "Content-Type:application/json" --body '{"key":"value"}'
```

PATCH request with headers and body:
```sh
cargo run -- --url <URL> --method patch --headers "Content-Type:application/json" --body '{"key":"value"}'
```

Concurrent requests with verbose output:
```sh
cargo run -- --url <URL> --users 50 --verbose
```

Save results to a file:
```sh
cargo run -- --url <URL> --output results.txt
```

## Output File

The output file will contain the details of each response received during the load test. Each line in the file will represent a `ResponseDetails` struct, which includes:
- `status`: The HTTP status code of the response.
- `time`: The time taken for the request in milliseconds.
- `timestamp`: The timestamp of when the request was made, in seconds since UNIX_EPOCH.

Example output:
```
ResponseDetails { status: 200, time: 844, timestamp: 1633024800 }
ResponseDetails { status: 200, time: 853, timestamp: 1633024801 }
ResponseDetails { status: 200, time: 853, timestamp: 1633024802 }
ResponseDetails { status: 200, time: 868, timestamp: 1633024803 }
ResponseDetails { status: 200, time: 870, timestamp: 1633024804 }
ResponseDetails { status: 200, time: 870, timestamp: 1633024805 }
ResponseDetails { status: 200, time: 888, timestamp: 1633024806 }
ResponseDetails { status: 200, time: 889, timestamp: 1633024807 }
ResponseDetails { status: 200, time: 893, timestamp: 1633024808 }
ResponseDetails { status: 200, time: 893, timestamp: 1633024809 }
```

## License

This project is licensed under the MIT License.
