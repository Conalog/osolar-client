make_client_with_fake_request <- function(response) {
  state <- new.env(parent = emptyenv())
  state$calls <- list()

  request_fn <- function(method, url, headers, query, body, timeout) {
    state$calls[[length(state$calls) + 1]] <- list(
      method = method,
      url = url,
      headers = headers,
      query = query,
      body = body,
      timeout = timeout
    )
    response
  }

  client <- osolar_client(
    api_key = "test-key",
    base_url = "https://example.com",
    request_fn = request_fn
  )

  list(client = client, state = state)
}

test_that("search_plants sends expected query and x-api-key header", {
  fake <- make_client_with_fake_request(list(
    status_code = 200,
    reason = "OK",
    body = '{"success":true,"data":{"features":[]}}'
  ))

  response <- search_plants(fake$client, q = "foo", field = "address", distance_km = 2)

  expect_true(response$success)
  expect_length(fake$state$calls, 1)
  call <- fake$state$calls[[1]]
  expect_identical(call$method, "GET")
  expect_identical(call$url, "https://example.com/v1/search")
  expect_identical(call$headers[["x-api-key"]], "test-key")
  expect_identical(call$query$q, "foo")
  expect_identical(call$query$field, "address")
  expect_identical(call$query$distance_km, 2)
})

test_that("search_plants rejects unsupported field", {
  fake <- make_client_with_fake_request(list(status_code = 200, reason = "OK", body = '{}'))

  expect_error(
    search_plants(fake$client, q = "foo", field = "name"),
    "`field` must be one of: business_number, address\\."
  )
})

test_that("link_plant supports keyword arguments", {
  fake <- make_client_with_fake_request(list(
    status_code = 200,
    reason = "OK",
    body = '{"success":true,"data":{"link_id":"link-1"}}'
  ))

  response <- link_plant(
    fake$client,
    plant_uuid = "plant-1",
    remark = "memo",
    link_id = "link-1"
  )

  expect_true(response$success)
  call <- fake$state$calls[[1]]
  expect_identical(call$method, "POST")
  expect_identical(call$url, "https://example.com/v1/links")
  expect_identical(call$body$plant_uuid, "plant-1")
  expect_identical(call$body$remark, "memo")
  expect_identical(call$body$link_id, "link-1")
})

test_that("link_plant rejects mixed payload styles", {
  fake <- make_client_with_fake_request(list(status_code = 200, reason = "OK", body = '{}'))

  expect_error(
    link_plant(
      fake$client,
      body = list(plant_uuid = "plant-1", remark = "memo"),
      plant_uuid = "plant-2",
      remark = "memo"
    ),
    "Use either `body` or keyword arguments, not both\\."
  )
})

test_that("get_plant_info URL-encodes link_id as a path segment", {
  fake <- make_client_with_fake_request(list(
    status_code = 200,
    reason = "OK",
    body = '{"success":true,"data":{"link_id":"abc/def ghi"}}'
  ))

  response <- get_plant_info(fake$client, "abc/def ghi")

  expect_true(response$success)
  call <- fake$state$calls[[1]]
  expect_identical(call$url, "https://example.com/v1/links/abc%2Fdef%20ghi")
})

test_that("monthly methods use endpoint-specific query keys", {
  fake <- make_client_with_fake_request(list(status_code = 200, reason = "OK", body = '{"success":true,"data":[]}'))

  generation <- get_monthly_generation(fake$client, "id", start_year = 2020, end_year = 2021)
  billing <- get_monthly_billing(fake$client, "id", start_year = 2020, end_year = 2021)

  expect_true(generation$success)
  expect_true(billing$success)

  generation_call <- fake$state$calls[[1]]
  billing_call <- fake$state$calls[[2]]

  expect_identical(generation_call$query$start_year, 2020)
  expect_identical(generation_call$query$end_year, 2021)
  expect_null(generation_call$query$startYear)
  expect_null(generation_call$query$endYear)

  expect_identical(billing_call$query$startYear, 2020)
  expect_identical(billing_call$query$endYear, 2021)
  expect_null(billing_call$query$start_year)
  expect_null(billing_call$query$end_year)
})

test_that("non-2xx response raises osolar_api_error with parsed response body", {
  fake <- make_client_with_fake_request(list(
    status_code = 403,
    reason = "Forbidden",
    body = '{"success":false,"message":"forbidden"}'
  ))

  expect_error(
    list_linked_plants(fake$client),
    class = "osolar_api_error"
  )

  error <- tryCatch(
    list_linked_plants(fake$client),
    osolar_api_error = function(e) e
  )

  expect_identical(error$status_code, 403)
  expect_identical(error$response_body$message, "forbidden")
  expect_identical(conditionMessage(error), "OSOLAR API error 403: Forbidden")
})

test_that("empty success response returns success envelope with NULL data", {
  fake <- make_client_with_fake_request(list(status_code = 204, reason = "No Content", body = ""))

  response <- list_linked_plants(fake$client)

  expect_identical(response, list(success = TRUE, data = NULL))
})

test_that("invalid JSON success response raises osolar_api_error", {
  fake <- make_client_with_fake_request(list(status_code = 200, reason = "OK", body = "<html>ok</html>"))

  error <- tryCatch(
    list_linked_plants(fake$client),
    osolar_api_error = function(e) e
  )

  expect_identical(error$status_code, 200)
  expect_identical(error$response_body, "<html>ok</html>")
  expect_identical(conditionMessage(error), "OSOLAR API error 200: Invalid JSON response")
})

test_that("non-object JSON success response raises osolar_api_error", {
  fake <- make_client_with_fake_request(list(status_code = 200, reason = "OK", body = "[1,2,3]"))

  error <- tryCatch(
    list_linked_plants(fake$client),
    osolar_api_error = function(e) e
  )

  expect_identical(error$status_code, 200)
  expect_identical(error$response_body, list(1L, 2L, 3L))
  expect_identical(conditionMessage(error), "OSOLAR API error 200: Unexpected JSON response type")
})

test_that("base_url defaults and disallows insecure non-localhost HTTP", {
  default_client <- osolar_client("test-key", request_fn = function(...) {
    list(status_code = 200, reason = "OK", body = '{"success":true,"data":{}}')
  })
  expect_identical(default_client$base_url, "https://openapi.osolar.io")

  expect_error(
    osolar_client("test-key", base_url = "http://example.com"),
    "`base_url` must use https:// \\(http:// is allowed only for localhost\\)\\."
  )
})

test_that("get_plant_contract normalizes legacy rec_fixed_contract object", {
  fake <- make_client_with_fake_request(list(
    status_code = 200,
    reason = "OK",
    body = '{"success":true,"data":{"ppa_type":"PPA","rec_trade_type":"fixed","rec_fixed_contract":{"target":"동서발전"}}}'
  ))

  response <- get_plant_contract(fake$client, "link-1")

  expect_true(response$success)
  expect_identical(response$data$rec_contracts, list(list(target = "동서발전", ess = FALSE)))
  expect_null(response$data$rec_fixed_contract)
})
