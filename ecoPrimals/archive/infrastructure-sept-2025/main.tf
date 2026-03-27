# Songbird Infrastructure as Code
# Production-ready deployment on AWS/GCP/Azure

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
  }

  backend "s3" {
    bucket = "songbird-terraform-state"
    key    = "infrastructure/terraform.tfstate"
    region = "us-west-2"
    
    # Enable state locking
    dynamodb_table = "songbird-terraform-locks"
    encrypt        = true
  }
}

# Variables
variable "environment" {
  description = "Environment name (dev, staging, production)"
  type        = string
  default     = "production"
}

variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "cluster_name" {
  description = "EKS cluster name"
  type        = string
  default     = "songbird-cluster"
}

variable "node_count" {
  description = "Number of worker nodes"
  type        = number
  default     = 3
}

variable "instance_type" {
  description = "EC2 instance type for worker nodes"
  type        = string
  default     = "t3.large"
}

variable "domain_name" {
  description = "Domain name for Songbird"
  type        = string
  default     = "songbird.ecoprimals.com"
}

# Provider configuration
provider "aws" {
  region = var.region
  
  default_tags {
    tags = {
      Project     = "Songbird"
      Environment = var.environment
      ManagedBy   = "Terraform"
      Owner       = "EcoPrimals"
    }
  }
}

# Data sources
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

# VPC and Networking
resource "aws_vpc" "songbird_vpc" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "songbird-vpc-${var.environment}"
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
  }
}

resource "aws_internet_gateway" "songbird_igw" {
  vpc_id = aws_vpc.songbird_vpc.id

  tags = {
    Name = "songbird-igw-${var.environment}"
  }
}

resource "aws_subnet" "songbird_public" {
  count = 2

  vpc_id                  = aws_vpc.songbird_vpc.id
  cidr_block              = "10.0.${count.index + 1}.0/24"
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "songbird-public-${count.index + 1}-${var.environment}"
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
    "kubernetes.io/role/elb"                    = "1"
  }
}

resource "aws_subnet" "songbird_private" {
  count = 2

  vpc_id            = aws_vpc.songbird_vpc.id
  cidr_block        = "10.0.${count.index + 10}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "songbird-private-${count.index + 1}-${var.environment}"
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
    "kubernetes.io/role/internal-elb"           = "1"
  }
}

# NAT Gateway
resource "aws_eip" "songbird_nat" {
  count  = 2
  domain = "vpc"

  tags = {
    Name = "songbird-nat-${count.index + 1}-${var.environment}"
  }
}

resource "aws_nat_gateway" "songbird_nat" {
  count = 2

  allocation_id = aws_eip.songbird_nat[count.index].id
  subnet_id     = aws_subnet.songbird_public[count.index].id

  tags = {
    Name = "songbird-nat-${count.index + 1}-${var.environment}"
  }

  depends_on = [aws_internet_gateway.songbird_igw]
}

# Route Tables
resource "aws_route_table" "songbird_public" {
  vpc_id = aws_vpc.songbird_vpc.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.songbird_igw.id
  }

  tags = {
    Name = "songbird-public-rt-${var.environment}"
  }
}

resource "aws_route_table" "songbird_private" {
  count = 2

  vpc_id = aws_vpc.songbird_vpc.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.songbird_nat[count.index].id
  }

  tags = {
    Name = "songbird-private-rt-${count.index + 1}-${var.environment}"
  }
}

resource "aws_route_table_association" "songbird_public" {
  count = 2

  subnet_id      = aws_subnet.songbird_public[count.index].id
  route_table_id = aws_route_table.songbird_public.id
}

resource "aws_route_table_association" "songbird_private" {
  count = 2

  subnet_id      = aws_subnet.songbird_private[count.index].id
  route_table_id = aws_route_table.songbird_private[count.index].id
}

# Security Groups
resource "aws_security_group" "songbird_eks_cluster" {
  name_prefix = "songbird-eks-cluster-"
  vpc_id      = aws_vpc.songbird_vpc.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "songbird-eks-cluster-sg-${var.environment}"
  }
}

resource "aws_security_group" "songbird_eks_nodes" {
  name_prefix = "songbird-eks-nodes-"
  vpc_id      = aws_vpc.songbird_vpc.id

  ingress {
    from_port = 0
    to_port   = 65535
    protocol  = "tcp"
    self      = true
  }

  ingress {
    from_port       = 1025
    to_port         = 65535
    protocol        = "tcp"
    security_groups = [aws_security_group.songbird_eks_cluster.id]
  }

  ingress {
    from_port       = 443
    to_port         = 443
    protocol        = "tcp"
    security_groups = [aws_security_group.songbird_eks_cluster.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "songbird-eks-nodes-sg-${var.environment}"
  }
}

# EKS Cluster
resource "aws_eks_cluster" "songbird" {
  name     = var.cluster_name
  role_arn = aws_iam_role.songbird_eks_cluster.arn

  vpc_config {
    subnet_ids              = concat(aws_subnet.songbird_public[*].id, aws_subnet.songbird_private[*].id)
    endpoint_private_access = true
    endpoint_public_access  = true
    security_group_ids      = [aws_security_group.songbird_eks_cluster.id]
  }

  enabled_cluster_log_types = ["api", "audit", "authenticator", "controllerManager", "scheduler"]

  depends_on = [
    aws_iam_role_policy_attachment.songbird_eks_cluster_policy,
    aws_iam_role_policy_attachment.songbird_eks_service_policy,
  ]

  tags = {
    Name = "songbird-cluster-${var.environment}"
  }
}

# EKS Node Group
resource "aws_eks_node_group" "songbird" {
  cluster_name    = aws_eks_cluster.songbird.name
  node_group_name = "songbird-nodes"
  node_role_arn   = aws_iam_role.songbird_eks_nodes.arn
  subnet_ids      = aws_subnet.songbird_private[*].id
  instance_types  = [var.instance_type]

  scaling_config {
    desired_size = var.node_count
    max_size     = var.node_count * 2
    min_size     = var.node_count
  }

  update_config {
    max_unavailable = 1
  }

  depends_on = [
    aws_iam_role_policy_attachment.songbird_eks_worker_node_policy,
    aws_iam_role_policy_attachment.songbird_eks_cni_policy,
    aws_iam_role_policy_attachment.songbird_eks_registry_policy,
  ]

  tags = {
    Name = "songbird-node-group-${var.environment}"
  }
}

# Outputs
output "cluster_id" {
  description = "EKS cluster ID"
  value       = aws_eks_cluster.songbird.id
}

output "cluster_arn" {
  description = "EKS cluster ARN"
  value       = aws_eks_cluster.songbird.arn
}

output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = aws_eks_cluster.songbird.endpoint
}

output "cluster_security_group_id" {
  description = "Security group ID attached to the EKS cluster"
  value       = aws_eks_cluster.songbird.vpc_config[0].cluster_security_group_id
}

output "kubectl_config" {
  description = "kubectl config as generated by the module"
  value = {
    cluster_name = aws_eks_cluster.songbird.name
    endpoint     = aws_eks_cluster.songbird.endpoint
    region       = var.region
  }
} 