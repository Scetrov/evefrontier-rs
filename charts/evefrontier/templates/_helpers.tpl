{{/*
Expand the name of the chart.
*/}}
{{- define "evefrontier.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "evefrontier.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "evefrontier.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "evefrontier.labels" -}}
helm.sh/chart: {{ include "evefrontier.chart" . }}
{{ include "evefrontier.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "evefrontier.selectorLabels" -}}
app.kubernetes.io/name: {{ include "evefrontier.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "evefrontier.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "evefrontier.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Route service labels
*/}}
{{- define "evefrontier.route.labels" -}}
{{ include "evefrontier.labels" . }}
app.kubernetes.io/component: route
{{- end }}

{{/*
Route service selector labels
*/}}
{{- define "evefrontier.route.selectorLabels" -}}
{{ include "evefrontier.selectorLabels" . }}
app.kubernetes.io/component: route
{{- end }}

{{/*
Scout-gates service labels
*/}}
{{- define "evefrontier.scoutGates.labels" -}}
{{ include "evefrontier.labels" . }}
app.kubernetes.io/component: scout-gates
{{- end }}

{{/*
Scout-gates service selector labels
*/}}
{{- define "evefrontier.scoutGates.selectorLabels" -}}
{{ include "evefrontier.selectorLabels" . }}
app.kubernetes.io/component: scout-gates
{{- end }}

{{/*
Scout-range service labels
*/}}
{{- define "evefrontier.scoutRange.labels" -}}
{{ include "evefrontier.labels" . }}
app.kubernetes.io/component: scout-range
{{- end }}

{{/*
Scout-range service selector labels
*/}}
{{- define "evefrontier.scoutRange.selectorLabels" -}}
{{ include "evefrontier.selectorLabels" . }}
app.kubernetes.io/component: scout-range
{{- end }}

{{/*
Get the image tag (defaults to chart appVersion)
*/}}
{{- define "evefrontier.route.imageTag" -}}
{{- .Values.route.image.tag | default .Chart.AppVersion }}
{{- end }}

{{- define "evefrontier.scoutGates.imageTag" -}}
{{- .Values.scoutGates.image.tag | default .Chart.AppVersion }}
{{- end }}

{{- define "evefrontier.scoutRange.imageTag" -}}
{{- .Values.scoutRange.image.tag | default .Chart.AppVersion }}
{{- end }}

{{/*
Create the PVC name
*/}}
{{- define "evefrontier.pvcName" -}}
{{- if .Values.persistence.existingClaim }}
{{- .Values.persistence.existingClaim }}
{{- else }}
{{- printf "%s-data" (include "evefrontier.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Container probes helper
*/}}
{{- define "evefrontier.probes" -}}
{{- if .Values.probes.liveness.enabled }}
livenessProbe:
  httpGet:
    path: {{ .Values.probes.liveness.path }}
    port: http
  initialDelaySeconds: {{ .Values.probes.liveness.initialDelaySeconds }}
  periodSeconds: {{ .Values.probes.liveness.periodSeconds }}
  timeoutSeconds: {{ .Values.probes.liveness.timeoutSeconds }}
  failureThreshold: {{ .Values.probes.liveness.failureThreshold }}
{{- end }}
{{- if .Values.probes.readiness.enabled }}
readinessProbe:
  httpGet:
    path: {{ .Values.probes.readiness.path }}
    port: http
  initialDelaySeconds: {{ .Values.probes.readiness.initialDelaySeconds }}
  periodSeconds: {{ .Values.probes.readiness.periodSeconds }}
  timeoutSeconds: {{ .Values.probes.readiness.timeoutSeconds }}
  failureThreshold: {{ .Values.probes.readiness.failureThreshold }}
{{- end }}
{{- end }}
