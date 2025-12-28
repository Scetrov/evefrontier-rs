# =============================================================================
# Terraform & Provider Requirements
# =============================================================================
# This file pins the Terraform version and required provider versions for
# the evefrontier-lambda module.
# =============================================================================

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0.0, < 6.0.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = ">= 2.4.0"
    }
  }
}
