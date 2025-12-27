# =============================================================================
# Lambda Function Resources
# =============================================================================
# Creates the three EveFrontier Lambda functions:
# - route: Pathfinding between solar systems
# - scout-gates: Find gate-connected neighbors
# - scout-range: Find systems within spatial range
# =============================================================================

# -----------------------------------------------------------------------------
# Local Values for Common Configuration
# -----------------------------------------------------------------------------

locals {
  common_tags = merge(var.tags, {
    Project     = var.project_name
    Environment = var.environment
    ManagedBy   = "terraform"
  })

  lambda_runtime = "provided.al2023"

  # Environment variables for all Lambda functions
  lambda_environment = {
    RUST_LOG = var.log_level
  }
}

# -----------------------------------------------------------------------------
# Data Sources: Package Lambda Binaries
# -----------------------------------------------------------------------------

# Route Lambda binary packaging
data "archive_file" "route" {
  count = can(regex(".*\\.zip$", var.route_binary_path)) ? 0 : 1

  type        = "zip"
  source_file = var.route_binary_path
  output_path = "${path.module}/.terraform/tmp/route-${var.environment}.zip"
}

# Scout Gates Lambda binary packaging
data "archive_file" "scout_gates" {
  count = can(regex(".*\\.zip$", var.scout_gates_binary_path)) ? 0 : 1

  type        = "zip"
  source_file = var.scout_gates_binary_path
  output_path = "${path.module}/.terraform/tmp/scout-gates-${var.environment}.zip"
}

# Scout Range Lambda binary packaging
data "archive_file" "scout_range" {
  count = can(regex(".*\\.zip$", var.scout_range_binary_path)) ? 0 : 1

  type        = "zip"
  source_file = var.scout_range_binary_path
  output_path = "${path.module}/.terraform/tmp/scout-range-${var.environment}.zip"
}

# -----------------------------------------------------------------------------
# Route Lambda Function
# -----------------------------------------------------------------------------

resource "aws_lambda_function" "route" {
  function_name = "${var.project_name}-route-${var.environment}"
  description   = "EveFrontier pathfinding - calculates routes between solar systems"

  role = aws_iam_role.lambda_execution.arn

  # Use pre-packaged zip if provided, otherwise use archive_file data source
  filename         = can(regex(".*\\.zip$", var.route_binary_path)) ? var.route_binary_path : data.archive_file.route[0].output_path
  source_code_hash = can(regex(".*\\.zip$", var.route_binary_path)) ? filebase64sha256(var.route_binary_path) : data.archive_file.route[0].output_base64sha256
  handler          = "bootstrap"
  runtime          = local.lambda_runtime
  architectures    = [var.lambda_architecture]

  memory_size = var.lambda_memory_mb
  timeout     = var.lambda_timeout_seconds

  reserved_concurrent_executions = var.lambda_reserved_concurrency >= 0 ? var.lambda_reserved_concurrency : null

  environment {
    variables = local.lambda_environment
  }

  # Optional VPC configuration
  dynamic "vpc_config" {
    for_each = var.vpc_config != null ? [var.vpc_config] : []
    content {
      subnet_ids         = vpc_config.value.subnet_ids
      security_group_ids = vpc_config.value.security_group_ids
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.route,
    aws_iam_role_policy.lambda_logging
  ]

  tags = merge(local.common_tags, {
    Name     = "${var.project_name}-route-${var.environment}"
    Function = "route"
  })
}

# -----------------------------------------------------------------------------
# Scout Gates Lambda Function
# -----------------------------------------------------------------------------

resource "aws_lambda_function" "scout_gates" {
  function_name = "${var.project_name}-scout-gates-${var.environment}"
  description   = "EveFrontier scout - finds gate-connected neighboring systems"

  role = aws_iam_role.lambda_execution.arn

  filename         = can(regex(".*\\.zip$", var.scout_gates_binary_path)) ? var.scout_gates_binary_path : data.archive_file.scout_gates[0].output_path
  source_code_hash = can(regex(".*\\.zip$", var.scout_gates_binary_path)) ? filebase64sha256(var.scout_gates_binary_path) : data.archive_file.scout_gates[0].output_base64sha256
  handler          = "bootstrap"
  runtime          = local.lambda_runtime
  architectures    = [var.lambda_architecture]

  memory_size = var.lambda_memory_mb
  timeout     = var.lambda_timeout_seconds

  reserved_concurrent_executions = var.lambda_reserved_concurrency >= 0 ? var.lambda_reserved_concurrency : null

  environment {
    variables = local.lambda_environment
  }

  dynamic "vpc_config" {
    for_each = var.vpc_config != null ? [var.vpc_config] : []
    content {
      subnet_ids         = vpc_config.value.subnet_ids
      security_group_ids = vpc_config.value.security_group_ids
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.scout_gates,
    aws_iam_role_policy.lambda_logging
  ]

  tags = merge(local.common_tags, {
    Name     = "${var.project_name}-scout-gates-${var.environment}"
    Function = "scout-gates"
  })
}

# -----------------------------------------------------------------------------
# Scout Range Lambda Function
# -----------------------------------------------------------------------------

resource "aws_lambda_function" "scout_range" {
  function_name = "${var.project_name}-scout-range-${var.environment}"
  description   = "EveFrontier scout - finds systems within spatial range using KD-tree index"

  role = aws_iam_role.lambda_execution.arn

  filename         = can(regex(".*\\.zip$", var.scout_range_binary_path)) ? var.scout_range_binary_path : data.archive_file.scout_range[0].output_path
  source_code_hash = can(regex(".*\\.zip$", var.scout_range_binary_path)) ? filebase64sha256(var.scout_range_binary_path) : data.archive_file.scout_range[0].output_base64sha256
  handler          = "bootstrap"
  runtime          = local.lambda_runtime
  architectures    = [var.lambda_architecture]

  memory_size = var.lambda_memory_mb
  timeout     = var.lambda_timeout_seconds

  reserved_concurrent_executions = var.lambda_reserved_concurrency >= 0 ? var.lambda_reserved_concurrency : null

  environment {
    variables = local.lambda_environment
  }

  dynamic "vpc_config" {
    for_each = var.vpc_config != null ? [var.vpc_config] : []
    content {
      subnet_ids         = vpc_config.value.subnet_ids
      security_group_ids = vpc_config.value.security_group_ids
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.scout_range,
    aws_iam_role_policy.lambda_logging
  ]

  tags = merge(local.common_tags, {
    Name     = "${var.project_name}-scout-range-${var.environment}"
    Function = "scout-range"
  })
}
