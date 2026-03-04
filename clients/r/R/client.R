osolar_client <- function(api_key, base_url = "https://openapi.osolar.io", timeout = 30, request_fn = NULL) {
  .assert_non_empty_string(api_key, "api_key")
  normalized_base_url <- .normalize_base_url(base_url)

  if (!is.numeric(timeout) || length(timeout) != 1 || is.na(timeout) || timeout <= 0) {
    stop("`timeout` must be a positive number.", call. = FALSE)
  }
  if (!is.null(request_fn) && !is.function(request_fn)) {
    stop("`request_fn` must be a function.", call. = FALSE)
  }

  structure(
    list(
      api_key = api_key,
      base_url = normalized_base_url,
      timeout = as.numeric(timeout),
      request_fn = request_fn
    ),
    class = "osolar_link_client"
  )
}

search_plants <- function(client, q, field, distance_km = NULL) {
  .assert_client(client)
  .assert_non_empty_string(q, "q")

  if (!is.character(field) || length(field) != 1 || !(field %in% c("business_number", "address"))) {
    stop("`field` must be one of: business_number, address.", call. = FALSE)
  }

  query <- list(q = q, field = field, distance_km = distance_km)
  .osolar_request(client, "GET", "/v1/search", query = query)
}

link_plant <- function(client, body = NULL, plant_uuid = NULL, remark = NULL, link_id = NULL) {
  .assert_client(client)

  has_keywords <- !is.null(plant_uuid) || !is.null(remark) || !is.null(link_id)
  if (!is.null(body) && has_keywords) {
    stop("Use either `body` or keyword arguments, not both.", call. = FALSE)
  }

  if (!is.null(body)) {
    if (!is.list(body) || is.null(body$plant_uuid) || is.null(body$remark)) {
      stop("`plant_uuid` and `remark` are required in `body`.", call. = FALSE)
    }
    payload <- body
  } else {
    if (is.null(plant_uuid) || is.null(remark)) {
      stop("`plant_uuid` and `remark` are required when `body` is not provided.", call. = FALSE)
    }
    payload <- list(plant_uuid = plant_uuid, remark = remark, link_id = link_id)
  }

  .osolar_request(client, "POST", "/v1/links", body = .drop_nulls(payload))
}

list_linked_plants <- function(client) {
  .assert_client(client)
  .osolar_request(client, "GET", "/v1/links")
}

get_plant_info <- function(client, link_id) {
  .assert_client(client)
  safe_link_id <- .encode_path_segment(link_id)
  .osolar_request(client, "GET", paste0("/v1/links/", safe_link_id))
}

get_plant_contract <- function(client, link_id) {
  .assert_client(client)
  safe_link_id <- .encode_path_segment(link_id)
  response <- .osolar_request(client, "GET", paste0("/v1/links/", safe_link_id, "/contract"))
  .normalize_plant_contract_response(response)
}

get_plant_documents <- function(client, link_id) {
  .assert_client(client)
  safe_link_id <- .encode_path_segment(link_id)
  .osolar_request(client, "GET", paste0("/v1/links/", safe_link_id, "/documents"))
}

get_plant_overview <- function(client, link_id) {
  .assert_client(client)
  safe_link_id <- .encode_path_segment(link_id)
  .osolar_request(client, "GET", paste0("/v1/links/", safe_link_id, "/overview"))
}

get_monthly_generation <- function(client, link_id, start_year = NULL, end_year = NULL) {
  .assert_client(client)
  safe_link_id <- .encode_path_segment(link_id)
  query <- list(start_year = start_year, end_year = end_year)
  .osolar_request(
    client,
    "GET",
    paste0("/v1/links/", safe_link_id, "/generation/monthly"),
    query = query
  )
}

get_monthly_billing <- function(client, link_id, start_year = NULL, end_year = NULL) {
  .assert_client(client)
  safe_link_id <- .encode_path_segment(link_id)
  query <- list(startYear = start_year, endYear = end_year)
  .osolar_request(
    client,
    "GET",
    paste0("/v1/links/", safe_link_id, "/billing/monthly"),
    query = query
  )
}

.assert_client <- function(client) {
  if (!inherits(client, "osolar_link_client")) {
    stop("`client` must be created by `osolar_client()`.", call. = FALSE)
  }
}

.assert_non_empty_string <- function(value, name) {
  if (!is.character(value) || length(value) != 1 || trimws(value) == "") {
    stop(paste0("`", name, "` must be a non-empty string."), call. = FALSE)
  }
}

.normalize_base_url <- function(base_url) {
  .assert_non_empty_string(base_url, "base_url")
  normalized <- sub("/+$", "", base_url)
  parsed <- httr::parse_url(normalized)

  if (is.null(parsed$scheme) || !(parsed$scheme %in% c("http", "https"))) {
    stop("`base_url` must start with http:// or https://.", call. = FALSE)
  }
  if (is.null(parsed$hostname) || parsed$hostname == "") {
    stop("`base_url` must include a hostname.", call. = FALSE)
  }
  if ((!is.null(parsed$query) && length(parsed$query) > 0) ||
      (!is.null(parsed$fragment) && nzchar(parsed$fragment))) {
    stop("`base_url` must not include query parameters or a fragment.", call. = FALSE)
  }

  if (parsed$scheme == "http" && !(parsed$hostname %in% c("127.0.0.1", "localhost", "::1"))) {
    stop("`base_url` must use https:// (http:// is allowed only for localhost).", call. = FALSE)
  }

  normalized
}

.drop_nulls <- function(x) {
  if (is.null(x) || length(x) == 0) {
    return(NULL)
  }
  x[!vapply(x, is.null, logical(1))]
}

.encode_path_segment <- function(value) {
  .assert_non_empty_string(value, "link_id")
  utils::URLencode(value, reserved = TRUE)
}

.osolar_request <- function(client, method, path, query = NULL, body = NULL) {
  request_fn <- client$request_fn
  if (is.null(request_fn)) {
    request_fn <- .default_request_fn
  }

  response <- request_fn(
    method = method,
    url = paste0(client$base_url, path),
    headers = list("x-api-key" = client$api_key),
    query = .drop_nulls(query),
    body = body,
    timeout = client$timeout
  )

  .parse_response(
    status_code = response$status_code,
    reason = response$reason,
    body = response$body
  )
}

.default_request_fn <- function(method, url, headers, query, body, timeout) {
  args <- list(
    verb = method,
    url = url,
    httr::add_headers(.headers = headers),
    httr::timeout(timeout),
    httr::config(followlocation = FALSE)
  )

  if (!is.null(query) && length(query) > 0) {
    args$query <- query
  }
  if (!is.null(body)) {
    args$body <- body
    args$encode <- "json"
  }

  response <- do.call(httr::VERB, args)

  list(
    status_code = httr::status_code(response),
    reason = httr::http_status(response)$reason,
    body = httr::content(response, as = "text", encoding = "UTF-8")
  )
}

.parse_response <- function(status_code, reason, body) {
  if (status_code < 200 || status_code >= 300) {
    parsed_error_body <- .parse_json_or_text(body)
    .signal_api_error(status_code, reason, parsed_error_body)
  }

  if (is.null(body) || identical(body, "") || identical(status_code, 204L) || identical(status_code, 204)) {
    return(list(success = TRUE, data = NULL))
  }

  parsed <- tryCatch(
    jsonlite::fromJSON(body, simplifyVector = FALSE),
    error = function(e) e
  )
  if (inherits(parsed, "error")) {
    .signal_api_error(status_code, "Invalid JSON response", body)
  }

  if (!is.list(parsed) || is.null(names(parsed))) {
    .signal_api_error(status_code, "Unexpected JSON response type", parsed)
  }

  parsed
}

.parse_json_or_text <- function(text) {
  if (is.null(text) || identical(text, "")) {
    return("")
  }
  parsed <- tryCatch(
    jsonlite::fromJSON(text, simplifyVector = FALSE),
    error = function(e) e
  )
  if (inherits(parsed, "error")) {
    return(text)
  }
  parsed
}

.signal_api_error <- function(status_code, reason, response_body) {
  error <- structure(
    list(
      message = paste0("OSOLAR API error ", status_code, ": ", reason),
      status_code = status_code,
      reason = reason,
      response_body = response_body
    ),
    class = c("osolar_api_error", "error", "condition")
  )
  stop(error)
}

.normalize_plant_contract_response <- function(response) {
  if (!is.list(response) || is.null(response$data) || !is.list(response$data)) {
    return(response)
  }

  data <- response$data
  contracts <- .coerce_contracts(data$rec_contracts)
  if (is.null(contracts)) {
    contracts <- .coerce_contracts(data$rec_fixed_contract)
  }
  if (is.null(contracts)) {
    contracts <- list()
  }

  data$rec_contracts <- lapply(contracts, .with_default_ess)
  data$rec_fixed_contract <- NULL
  response$data <- data
  response
}

.coerce_contracts <- function(value) {
  if (is.null(value) || !is.list(value)) {
    return(NULL)
  }

  if (!is.null(names(value))) {
    return(list(value))
  }

  value
}

.with_default_ess <- function(item) {
  if (!is.list(item)) {
    return(item)
  }
  if (is.null(item$ess)) {
    item$ess <- FALSE
  }
  item
}
