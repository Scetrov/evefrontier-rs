# =============================================================================
# Input Variables for evefrontier-lambda Module
# =============================================================================
# All configurable parameters for deploying the EVE Frontier Lambda functions.
# Variables are grouped by category for clarity.
# =============================================================================

# -----------------------------------------------------------------------------
# Core Configuration
# -----------------------------------------------------------------------------

variable "environment" {
  description = "Deployment environment name (e.g., dev, staging, prod). Used for resource naming and tagging."
  type        = string
  default     = "dev"

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.environment))
    error_message = "Environment must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "project_name" {
  description = "Project name prefix for all resources. Defaults to 'evefrontier'."
  type        = string
  default     = "evefrontier"

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.project_name))
    error_message = "Project name must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "tags" {
  description = "Additional tags to apply to all resources. Environment and Project tags are added automatically."
  type        = map(string)
  default     = {}
}

# -----------------------------------------------------------------------------
# Lambda Function Configuration
# -----------------------------------------------------------------------------

variable "lambda_memory_mb" {
  description = "Memory allocation in MB for Lambda functions. Bundled dataset requires ~256MB minimum; 512MB+ recommended for better performance."
  type        = number
  default     = 512

  validation {
    condition     = var.lambda_memory_mb >= 128 && var.lambda_memory_mb <= 10240
    error_message = "Lambda memory must be between 128 MB and 10240 MB."
  }
}

variable "lambda_timeout_seconds" {
  description = "Timeout in seconds for Lambda functions. Route calculations typically complete in <1s."
  type        = number
  default     = 10

  validation {
    condition     = var.lambda_timeout_seconds >= 1 && var.lambda_timeout_seconds <= 900
    error_message = "Lambda timeout must be between 1 and 900 seconds."
  }
}

variable "lambda_architecture" {
  description = "CPU architecture for Lambda functions. arm64 offers better price/performance for Rust."
  type        = string
  default     = "arm64"

  validation {
    condition     = contains(["arm64", "x86_64"], var.lambda_architecture)
    error_message = "Lambda architecture must be either 'arm64' or 'x86_64'."
  }
}

variable "lambda_reserved_concurrency" {
  description = "Reserved concurrent executions for each Lambda. Set to -1 for unreserved (auto-scaling)."
  type        = number
  default     = -1

  validation {
    condition     = var.lambda_reserved_concurrency >= -1
    error_message = "Reserved concurrency must be -1 (unreserved) or a non-negative integer."
  }
}

# -----------------------------------------------------------------------------
# Lambda Binary Paths
# -----------------------------------------------------------------------------

variable "route_binary_path" {
  description = "Path to the built 'bootstrap' binary for the route Lambda function. May be either a raw 'bootstrap' file or a pre-packaged '.zip' archive. When providing a '.zip', it MUST contain a file named 'bootstrap' in the root of the archive, as required by the AWS Lambda custom runtime."
  type        = string

  validation {
    condition     = can(regex(".*bootstrap$", var.route_binary_path)) || can(regex(".*\\.zip$", var.route_binary_path))
    error_message = "Binary path must end with 'bootstrap' (raw binary) or '.zip' (pre-packaged). If using '.zip', ensure the archive contains a 'bootstrap' file in its root."
  }
}

variable "scout_gates_binary_path" {
  description = "Path to the built 'bootstrap' binary for the scout-gates Lambda function. May be either a raw 'bootstrap' file or a pre-packaged '.zip' archive. When providing a '.zip', it MUST contain a file named 'bootstrap' in the root of the archive, as required by the AWS Lambda custom runtime."
  type        = string

  validation {
    condition     = can(regex(".*bootstrap$", var.scout_gates_binary_path)) || can(regex(".*\\.zip$", var.scout_gates_binary_path))
    error_message = "Binary path must end with 'bootstrap' (raw binary) or '.zip' (pre-packaged). If using '.zip', ensure the archive contains a 'bootstrap' file in its root."
  }
}

variable "scout_range_binary_path" {
  description = "Path to the built 'bootstrap' binary for the scout-range Lambda function. May be either a raw 'bootstrap' file or a pre-packaged '.zip' archive. When providing a '.zip', it MUST contain a file named 'bootstrap' in the root of the archive, as required by the AWS Lambda custom runtime."
  type        = string

  validation {
    condition     = can(regex(".*bootstrap$", var.scout_range_binary_path)) || can(regex(".*\\.zip$", var.scout_range_binary_path))
    error_message = "Binary path must end with 'bootstrap' (raw binary) or '.zip' (pre-packaged). If using '.zip', ensure the archive contains a 'bootstrap' file in its root."
  }
}

# -----------------------------------------------------------------------------
# CloudWatch Logging Configuration
# -----------------------------------------------------------------------------

variable "log_retention_days" {
  description = "CloudWatch log retention period in days. Set to 0 for indefinite retention."
  type        = number
  default     = 30

  validation {
    condition     = contains([0, 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653], var.log_retention_days)
    error_message = "Log retention must be a valid CloudWatch Logs retention period."
  }
}

variable "log_level" {
  description = "Log level for Lambda functions. Controls tracing verbosity."
  type        = string
  default     = "info"

  validation {
    condition     = contains(["trace", "debug", "info", "warn", "error"], var.log_level)
    error_message = "Log level must be one of: trace, debug, info, warn, error."
  }
}

# -----------------------------------------------------------------------------
# API Gateway Configuration
# -----------------------------------------------------------------------------

variable "api_stage_name" {
  description = "API Gateway stage name (e.g., 'v1', 'prod'). Forms part of the API URL."
  type        = string
  default     = "v1"

  validation {
    condition     = can(regex("^[a-zA-Z0-9_-]+$", var.api_stage_name))
    error_message = "Stage name must contain only alphanumeric characters, hyphens, and underscores."
  }
}

variable "cors_allowed_origins" {
  description = "List of allowed CORS origins. Use ['*'] only for development; specify explicit origins for production."
  type        = list(string)
  default     = ["*"]

  validation {
    # Using '*' is only allowed for explicitly non-production environments.
    # This protects against overly permissive CORS in production-like environments.
    condition     = !contains(var.cors_allowed_origins, "*") || contains(["dev", "local", "test", "staging"], lower(var.environment))
    error_message = "Using '*' for cors_allowed_origins is only allowed when environment is one of: dev, local, test, staging. Specify explicit allowed origins for other environments."
  }
}

variable "cors_allowed_methods" {
  description = "List of allowed HTTP methods for CORS."
  type        = list(string)
  default     = ["POST", "OPTIONS"]
}

variable "cors_allowed_headers" {
  description = "List of allowed headers for CORS requests."
  type        = list(string)
  default     = ["Content-Type", "Authorization", "X-Amz-Date", "X-Api-Key", "X-Amz-Security-Token"]
}

variable "cors_max_age_seconds" {
  description = "How long browsers should cache CORS preflight responses."
  type        = number
  default     = 300

  validation {
    condition     = var.cors_max_age_seconds >= 0 && var.cors_max_age_seconds <= 86400
    error_message = "CORS max age must be between 0 and 86400 seconds (24 hours)."
  }
}

variable "throttling_burst_limit" {
  description = "API Gateway throttling burst limit (requests per second burst)."
  type        = number
  default     = 100

  validation {
    condition     = var.throttling_burst_limit > 0
    error_message = "Throttling burst limit must be greater than 0."
  }
}

variable "throttling_rate_limit" {
  description = "API Gateway throttling rate limit (requests per second sustained). Must not exceed burst limit."
  type        = number
  default     = 50

  validation {
    condition     = var.throttling_rate_limit > 0 && var.throttling_rate_limit <= var.throttling_burst_limit
    error_message = "Throttling rate limit must be greater than 0 and must not exceed the throttling burst limit."
  }
}

# -----------------------------------------------------------------------------
# Optional: VPC Configuration (for private deployments)
# -----------------------------------------------------------------------------

variable "vpc_config" {
  description = "Optional VPC configuration for Lambda functions. Set to null for public deployment."
  type = object({
    subnet_ids         = list(string)
    security_group_ids = list(string)
  })
  default = null
}
