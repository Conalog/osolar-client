\name{osolar_client}
\alias{osolar_client}
\alias{search_plants}
\alias{link_plant}
\alias{list_linked_plants}
\alias{get_plant_info}
\alias{get_plant_contract}
\alias{get_plant_documents}
\alias{get_plant_overview}
\alias{get_monthly_generation}
\alias{get_monthly_billing}
\title{OSOLAR-LINK Open API Client Functions}
\description{
Create an OSOLAR-LINK API client and call supported endpoints.
}
\usage{
osolar_client(api_key, base_url = "https://openapi.osolar.io", timeout = 30, request_fn = NULL)

search_plants(client, q, field, distance_km = NULL)

link_plant(client, body = NULL, plant_uuid = NULL, remark = NULL, link_id = NULL)

list_linked_plants(client)

get_plant_info(client, link_id)

get_plant_contract(client, link_id)

get_plant_documents(client, link_id)

get_plant_overview(client, link_id)

get_monthly_generation(client, link_id, start_year = NULL, end_year = NULL)

get_monthly_billing(client, link_id, start_year = NULL, end_year = NULL)
}
\arguments{
  \item{api_key}{API key used in the \code{x-api-key} header.}
  \item{base_url}{API base URL. Defaults to \code{https://openapi.osolar.io}.}
  \item{timeout}{Request timeout in seconds.}
  \item{request_fn}{Optional custom request function for testing.}
  \item{client}{Client object created by \code{osolar_client()}.}
  \item{q}{Search query text.}
  \item{field}{Search field. One of \code{"business_number"} or \code{"address"}.}
  \item{distance_km}{Optional search distance in kilometers.}
  \item{body}{Optional full request body for \code{link_plant()}.}
  \item{plant_uuid}{Plant UUID for link creation.}
  \item{remark}{Link request remark.}
  \item{link_id}{Link identifier for detail endpoints or optional custom ID for link creation.}
  \item{start_year}{Optional start year filter.}
  \item{end_year}{Optional end year filter.}
}
\value{
All endpoint functions return a parsed API response as an R list.
When the API returns an error status or invalid success payload, an
\code{osolar_api_error} condition is raised.
}
\examples{
\dontrun{
client <- osolar_client(Sys.getenv("OSOLAR_API_KEY"))
links <- list_linked_plants(client)
print(links$data)
}
}
