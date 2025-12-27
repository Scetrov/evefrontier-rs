# =============================================================================
# CloudWatch Log Groups for Lambda Functions
# =============================================================================
# Creates log groups with configurable retention for each Lambda function.
# Log groups are created before Lambda functions to ensure proper permissions.
# =============================================================================

# -----------------------------------------------------------------------------
# Route Lambda Log Group
# -----------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "route" {
  name              = "/aws/lambda/${var.project_name}-route-${var.environment}"
  retention_in_days = var.log_retention_days > 0 ? var.log_retention_days : null

  tags = merge(var.tags, {
    Name        = "${var.project_name}-route-logs-${var.environment}"
    Project     = var.project_name
    Environment = var.environment
    Function    = "route"
  })
}

# -----------------------------------------------------------------------------
# Scout Gates Lambda Log Group
# -----------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "scout_gates" {
  name              = "/aws/lambda/${var.project_name}-scout-gates-${var.environment}"
  retention_in_days = var.log_retention_days > 0 ? var.log_retention_days : null

  tags = merge(var.tags, {
    Name        = "${var.project_name}-scout-gates-logs-${var.environment}"
    Project     = var.project_name
    Environment = var.environment
    Function    = "scout-gates"
  })
}

# -----------------------------------------------------------------------------
# Scout Range Lambda Log Group
# -----------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "scout_range" {
  name              = "/aws/lambda/${var.project_name}-scout-range-${var.environment}"
  retention_in_days = var.log_retention_days > 0 ? var.log_retention_days : null

  tags = merge(var.tags, {
    Name        = "${var.project_name}-scout-range-logs-${var.environment}"
    Project     = var.project_name
    Environment = var.environment
    Function    = "scout-range"
  })
}
