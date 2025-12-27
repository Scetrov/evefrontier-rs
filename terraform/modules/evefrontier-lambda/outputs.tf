# =============================================================================
# Module Outputs
# =============================================================================
# Exports useful values for consumers of this module.
# =============================================================================

# -----------------------------------------------------------------------------
# API Gateway Outputs
# -----------------------------------------------------------------------------

output "api_endpoint" {
  description = "Complete invoke URL for the API Gateway (includes stage)"
  value       = aws_apigatewayv2_stage.main.invoke_url
}

output "api_id" {
  description = "API Gateway API ID"
  value       = aws_apigatewayv2_api.main.id
}

output "api_execution_arn" {
  description = "API Gateway execution ARN (for IAM policies)"
  value       = aws_apigatewayv2_api.main.execution_arn
}

# -----------------------------------------------------------------------------
# Lambda Function Outputs
# -----------------------------------------------------------------------------

output "route_function_name" {
  description = "Name of the route Lambda function"
  value       = aws_lambda_function.route.function_name
}

output "route_function_arn" {
  description = "ARN of the route Lambda function"
  value       = aws_lambda_function.route.arn
}

output "route_invoke_arn" {
  description = "Invoke ARN of the route Lambda function"
  value       = aws_lambda_function.route.invoke_arn
}

output "scout_gates_function_name" {
  description = "Name of the scout-gates Lambda function"
  value       = aws_lambda_function.scout_gates.function_name
}

output "scout_gates_function_arn" {
  description = "ARN of the scout-gates Lambda function"
  value       = aws_lambda_function.scout_gates.arn
}

output "scout_gates_invoke_arn" {
  description = "Invoke ARN of the scout-gates Lambda function"
  value       = aws_lambda_function.scout_gates.invoke_arn
}

output "scout_range_function_name" {
  description = "Name of the scout-range Lambda function"
  value       = aws_lambda_function.scout_range.function_name
}

output "scout_range_function_arn" {
  description = "ARN of the scout-range Lambda function"
  value       = aws_lambda_function.scout_range.arn
}

output "scout_range_invoke_arn" {
  description = "Invoke ARN of the scout-range Lambda function"
  value       = aws_lambda_function.scout_range.invoke_arn
}

# -----------------------------------------------------------------------------
# IAM Outputs
# -----------------------------------------------------------------------------

output "lambda_execution_role_arn" {
  description = "ARN of the Lambda execution IAM role"
  value       = aws_iam_role.lambda_execution.arn
}

output "lambda_execution_role_name" {
  description = "Name of the Lambda execution IAM role"
  value       = aws_iam_role.lambda_execution.name
}

# -----------------------------------------------------------------------------
# CloudWatch Log Group Outputs
# -----------------------------------------------------------------------------

output "log_group_names" {
  description = "Map of function names to CloudWatch Log Group names"
  value = {
    route       = aws_cloudwatch_log_group.route.name
    scout_gates = aws_cloudwatch_log_group.scout_gates.name
    scout_range = aws_cloudwatch_log_group.scout_range.name
    api_gateway = aws_cloudwatch_log_group.api_gateway.name
  }
}

output "log_group_arns" {
  description = "Map of function names to CloudWatch Log Group ARNs"
  value = {
    route       = aws_cloudwatch_log_group.route.arn
    scout_gates = aws_cloudwatch_log_group.scout_gates.arn
    scout_range = aws_cloudwatch_log_group.scout_range.arn
    api_gateway = aws_cloudwatch_log_group.api_gateway.arn
  }
}

# -----------------------------------------------------------------------------
# Endpoint URLs (Convenience)
# -----------------------------------------------------------------------------

output "endpoint_urls" {
  description = "Complete endpoint URLs for each API route"
  value = {
    route       = "${aws_apigatewayv2_stage.main.invoke_url}/route"
    scout_gates = "${aws_apigatewayv2_stage.main.invoke_url}/scout-gates"
    scout_range = "${aws_apigatewayv2_stage.main.invoke_url}/scout-range"
  }
}
