# =============================================================================
# API Gateway HTTP API (v2)
# =============================================================================
# Creates an HTTP API with routes for the three Lambda functions.
# HTTP API v2 offers lower latency and cost compared to REST API.
# =============================================================================

# -----------------------------------------------------------------------------
# HTTP API
# -----------------------------------------------------------------------------

resource "aws_apigatewayv2_api" "main" {
  name          = "${var.project_name}-api-${var.environment}"
  description   = "EVE Frontier API - Pathfinding and scouting for EVE Frontier"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins     = var.cors_allowed_origins
    allow_methods     = var.cors_allowed_methods
    allow_headers     = var.cors_allowed_headers
    max_age           = var.cors_max_age_seconds
    allow_credentials = false
  }

  tags = merge(var.tags, {
    Name        = "${var.project_name}-api-${var.environment}"
    Project     = var.project_name
    Environment = var.environment
  })
}

# -----------------------------------------------------------------------------
# API Stage
# -----------------------------------------------------------------------------

resource "aws_apigatewayv2_stage" "main" {
  api_id      = aws_apigatewayv2_api.main.id
  name        = var.api_stage_name
  auto_deploy = true

  default_route_settings {
    throttling_burst_limit = var.throttling_burst_limit
    throttling_rate_limit  = var.throttling_rate_limit
  }

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.api_gateway.arn
    format = jsonencode({
      requestId        = "$context.requestId"
      ip               = "$context.identity.sourceIp"
      requestTime      = "$context.requestTime"
      httpMethod       = "$context.httpMethod"
      routeKey         = "$context.routeKey"
      status           = "$context.status"
      responseLength   = "$context.responseLength"
      integrationError = "$context.integrationErrorMessage"
      latency          = "$context.responseLatency"
    })
  }

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-api-stage-${var.environment}"
  })
}

# -----------------------------------------------------------------------------
# API Gateway Log Group
# -----------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "api_gateway" {
  name              = "/aws/apigateway/${var.project_name}-api-${var.environment}"
  retention_in_days = var.log_retention_days > 0 ? var.log_retention_days : null

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-api-logs-${var.environment}"
  })
}

# -----------------------------------------------------------------------------
# Lambda Integrations
# -----------------------------------------------------------------------------

# Route Lambda Integration
resource "aws_apigatewayv2_integration" "route" {
  api_id                 = aws_apigatewayv2_api.main.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.route.invoke_arn
  integration_method     = "POST"
  payload_format_version = "2.0"
}

# Scout Gates Lambda Integration
resource "aws_apigatewayv2_integration" "scout_gates" {
  api_id                 = aws_apigatewayv2_api.main.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.scout_gates.invoke_arn
  integration_method     = "POST"
  payload_format_version = "2.0"
}

# Scout Range Lambda Integration
resource "aws_apigatewayv2_integration" "scout_range" {
  api_id                 = aws_apigatewayv2_api.main.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.scout_range.invoke_arn
  integration_method     = "POST"
  payload_format_version = "2.0"
}

# -----------------------------------------------------------------------------
# API Routes
# -----------------------------------------------------------------------------

# POST /route
resource "aws_apigatewayv2_route" "route" {
  api_id    = aws_apigatewayv2_api.main.id
  route_key = "POST /route"
  target    = "integrations/${aws_apigatewayv2_integration.route.id}"
}

# POST /scout-gates
resource "aws_apigatewayv2_route" "scout_gates" {
  api_id    = aws_apigatewayv2_api.main.id
  route_key = "POST /scout-gates"
  target    = "integrations/${aws_apigatewayv2_integration.scout_gates.id}"
}

# POST /scout-range
resource "aws_apigatewayv2_route" "scout_range" {
  api_id    = aws_apigatewayv2_api.main.id
  route_key = "POST /scout-range"
  target    = "integrations/${aws_apigatewayv2_integration.scout_range.id}"
}

# -----------------------------------------------------------------------------
# Lambda Permissions for API Gateway
# -----------------------------------------------------------------------------
# Source ARN is scoped to specific stage and HTTP method for least privilege.
# Format: {execution_arn}/{stage}/{method}/{path}
# Statement IDs are descriptive to aid debugging and auditing in AWS Console.
# -----------------------------------------------------------------------------

resource "aws_lambda_permission" "route" {
  statement_id  = "AllowAPIGatewayInvokeRoute"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.route.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.main.execution_arn}/${var.api_stage_name}/POST/route"
}

resource "aws_lambda_permission" "scout_gates" {
  statement_id  = "AllowAPIGatewayInvokeScoutGates"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.scout_gates.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.main.execution_arn}/${var.api_stage_name}/POST/scout-gates"
}

resource "aws_lambda_permission" "scout_range" {
  statement_id  = "AllowAPIGatewayInvokeScoutRange"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.scout_range.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.main.execution_arn}/${var.api_stage_name}/POST/scout-range"
}
