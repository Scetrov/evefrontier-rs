# =============================================================================
# IAM Resources for Lambda Functions
# =============================================================================
# Creates the IAM execution role and policies for Lambda functions.
# Follows least-privilege principle - only CloudWatch Logs permissions.
# =============================================================================

# -----------------------------------------------------------------------------
# Lambda Execution Role
# -----------------------------------------------------------------------------

resource "aws_iam_role" "lambda_execution" {
  name = "${var.project_name}-lambda-execution-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
        Action = "sts:AssumeRole"
      }
    ]
  })

  tags = merge(var.tags, {
    Name        = "${var.project_name}-lambda-execution-${var.environment}"
    Project     = var.project_name
    Environment = var.environment
  })
}

# -----------------------------------------------------------------------------
# CloudWatch Logs Policy (Least Privilege)
# -----------------------------------------------------------------------------
# Log group creation is handled by Terraform (cloudwatch.tf).
# Lambda only needs permissions to write to existing log groups.
# If log groups are accidentally deleted, run `terraform apply` to recreate.
#
# The `:*` suffix on log group ARNs is required for log stream operations.
# CloudWatch Logs ARN format: arn:aws:logs:region:account:log-group:name:*
# This allows Lambda to create log streams (with any name) and write events
# to them within the specified log groups. This is the standard AWS pattern
# for Lambda logging - Lambda auto-generates stream names based on invocation.
# -----------------------------------------------------------------------------

resource "aws_iam_role_policy" "lambda_logging" {
  name = "${var.project_name}-lambda-logging-${var.environment}"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "CloudWatchLogsWrite"
        Effect = "Allow"
        Action = [
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        # The :* suffix allows operations on log streams within the log group.
        # Lambda creates streams named: YYYY/MM/DD/[$LATEST|version]/instance-id
        Resource = [
          "${aws_cloudwatch_log_group.route.arn}:*",
          "${aws_cloudwatch_log_group.scout_gates.arn}:*",
          "${aws_cloudwatch_log_group.scout_range.arn}:*"
        ]
      }
    ]
  })
}

# -----------------------------------------------------------------------------
# Optional: VPC Policy (only if VPC config is provided)
# -----------------------------------------------------------------------------
# NOTE: Resource = "*" is required for Lambda VPC ENI operations.
# AWS Lambda dynamically creates Elastic Network Interfaces (ENIs) at runtime
# when functions are configured to access VPC resources. The ENI IDs are not
# known until invocation time, making resource-level restrictions impossible.
# This is a documented AWS limitation and a known exception to least-privilege.
# See: https://docs.aws.amazon.com/lambda/latest/dg/configuration-vpc.html
# -----------------------------------------------------------------------------

resource "aws_iam_role_policy" "lambda_vpc" {
  count = var.vpc_config != null ? 1 : 0

  name = "${var.project_name}-lambda-vpc-${var.environment}"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "VPCNetworkInterfaceAccess"
        Effect = "Allow"
        Action = [
          "ec2:CreateNetworkInterface",
          "ec2:DescribeNetworkInterfaces",
          "ec2:DeleteNetworkInterface",
          "ec2:AssignPrivateIpAddresses",
          "ec2:UnassignPrivateIpAddresses"
        ]
        # Wildcard required: Lambda creates ENIs dynamically at runtime.
        # ENI resource ARNs are not predictable before function invocation.
        Resource = "*"
      }
    ]
  })
}
