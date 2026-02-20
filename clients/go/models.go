package osolar

type ApiResponse[T any] struct {
	Success bool    `json:"success"`
	Message *string `json:"message,omitempty"`
	Data    *T      `json:"data,omitempty"`
}

func (a *ApiResponse[T]) markSuccess() {
	if a != nil {
		a.Success = true
	}
}

type GeoPoint struct {
	Type        string    `json:"type,omitempty"`
	Coordinates []float64 `json:"coordinates"`
}

type PlantOwner struct {
	BusinessNumber     string `json:"business_number"`
	FirmName           string `json:"firm_name"`
	FirmAddress        string `json:"firm_address"`
	RepresentativeName string `json:"representative_name"`
}

type PlantFeatureProperties struct {
	PlantUUID     string     `json:"plant_uuid"`
	PlantName     string     `json:"plant_name"`
	PlantAddress  string     `json:"plant_address"`
	PlantCapacity float64    `json:"plant_capacity"`
	PlantOwner    PlantOwner `json:"plant_owner"`
}

type PlantFeature struct {
	Type       string                 `json:"type,omitempty"`
	Geometry   GeoPoint               `json:"geometry"`
	Properties PlantFeatureProperties `json:"properties"`
}

type PlantGeoJSONResponse struct {
	Type     string         `json:"type,omitempty"`
	Features []PlantFeature `json:"features"`
}

type PlantLinkRequest struct {
	PlantUUID string  `json:"plant_uuid"`
	LinkID    *string `json:"link_id,omitempty"`
	Remark    string  `json:"remark"`
}

type PlantLinkResponse struct {
	LinkID    string `json:"link_id"`
	CreatedAt string `json:"created_at"`
}

type PlantLinkListResponse struct {
	LinkID       string  `json:"link_id"`
	PlantName    string  `json:"plant_name"`
	PlantAddress *string `json:"plant_address,omitempty"`
	Remark       *string `json:"remark,omitempty"`
	CreatedAt    *string `json:"created_at,omitempty"`
}

type PlantInfoResponse struct {
	LinkID         string     `json:"link_id"`
	PlantName      string     `json:"plant_name"`
	PlantAddress   string     `json:"plant_address"`
	PlantCapacity  string     `json:"plant_capacity"`
	PlantCertified bool       `json:"plant_certified"`
	PlantGeometry  GeoPoint   `json:"plant_geometry"`
	PlantOwner     PlantOwner `json:"plant_owner"`
}

type RecFixedContractInfo struct {
	ESS           bool    `json:"ess"`
	Target        string  `json:"target"`
	PriceType     *string `json:"price_type,omitempty"`
	Price         *int    `json:"price,omitempty"`
	StartDate     *string `json:"start_date,omitempty"`
	EndDate       *string `json:"end_date,omitempty"`
	ContractYears *int    `json:"contract_years,omitempty"`
}

type PlantContractResponse struct {
	PPAType      string                 `json:"ppa_type"`
	RecTradeType string                 `json:"rec_trade_type"`
	RecContracts []RecFixedContractInfo `json:"rec_contracts"`
}

type DocumentResponse struct {
	DocumentID   string `json:"document_id"`
	DocumentCode string `json:"document_code"`
	DocumentName string `json:"document_name"`
	FileName     string `json:"file_name"`
	DownloadURL  string `json:"download_url"`
}

type BillingAmountResponse struct {
	BillingMonth     string `json:"billing_month"`
	SMPBillingAmount int    `json:"smp_billing_amount"`
	RECBillingAmount *int   `json:"rec_billing_amount"`
}

type GenerationAmountResponse struct {
	GenerationMonth        string `json:"generation_month"`
	GenerationAmount       int    `json:"generation_amount"`
	NearbyGenerationAmount int    `json:"nearby_generation_amount"`
}

type TaskDetail struct {
	Date    *string `json:"date,omitempty"`
	Name    string  `json:"name"`
	Type    string  `json:"type"`
	Status  string  `json:"status"`
	Summary string  `json:"summary"`
	Value   string  `json:"value"`
}

type PlantOverviewResponse struct {
	LinkID         string                  `json:"link_id"`
	PlantName      string                  `json:"plant_name"`
	BillingSummary []BillingAmountResponse `json:"billing_summary"`
	RecentTasks    []TaskDetail            `json:"recent_tasks"`
}

type SearchPlantsParams struct {
	Q          string
	Field      string
	DistanceKM *float64
}

type MonthlyGenerationParams struct {
	StartYear *int
	EndYear   *int
}

type MonthlyBillingParams struct {
	StartYear *int
	EndYear   *int
}
