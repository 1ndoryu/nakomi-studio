<#
[055A-3] Sincroniza el catalogo de servicios/planes por API admin real a partir de la
propuesta JSON, sin usar fixtures ni escrituras directas a BD.
Gotcha: Nakomi tiene slugs web duplicados (`diseno-de-sitios-web` activo y `diseno-web`
inactivo), por eso el sync debe preferir el servicio activo antes de caer a aliases legacy.
[055A-4] No fusionar `ecommerce` y `marketing-digital` para acomodar fallbacks del frontend:
el catalogo real manda y los slugs nuevos deben resolverse como strings completos, no como
caracteres indexados por PowerShell.
Pendiente: las imagenes se cargan aparte; si la propuesta no trae media, el script preserva
la existente y omite campos vacios para no pisar el CMS.
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$BaseUrl,

    [Parameter(Mandatory = $true)]
    [string]$JwtSecret,

    [string]$UserId = '00000000-0000-0000-0000-000000000001',

    [string]$ProposalPath = 'Agente/documentacion/panel/propuesta-servicios-planes-2026-05-05.json',

    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir

function ConvertTo-Base64Url {
    param([byte[]]$Bytes)

    $encoded = [Convert]::ToBase64String($Bytes)
    $encoded = $encoded.TrimEnd('=')
    $encoded = $encoded.Replace('+', '-')
    $encoded = $encoded.Replace('/', '_')
    return $encoded
}

function New-AdminJwt {
    param(
        [string]$Secret,
        [string]$Subject
    )

    $header = @{ alg = 'HS256'; typ = 'JWT' } | ConvertTo-Json -Compress
    $epoch = [DateTimeOffset]::UtcNow.AddYears(10).ToUnixTimeSeconds()
    $claims = @{
        sub = $Subject
        role = 'admin'
        effective_role = 'admin'
        exp = $epoch
    } | ConvertTo-Json -Compress

    $headerSegment = ConvertTo-Base64Url ([Text.Encoding]::UTF8.GetBytes($header))
    $claimsSegment = ConvertTo-Base64Url ([Text.Encoding]::UTF8.GetBytes($claims))
    $unsignedToken = "$headerSegment.$claimsSegment"

    $hmac = [System.Security.Cryptography.HMACSHA256]::new([Text.Encoding]::UTF8.GetBytes($Secret))
    try {
        $signatureBytes = $hmac.ComputeHash([Text.Encoding]::UTF8.GetBytes($unsignedToken))
    }
    finally {
        $hmac.Dispose()
    }

    $signatureSegment = ConvertTo-Base64Url $signatureBytes
    return "$unsignedToken.$signatureSegment"
}

function Invoke-CmsJson {
    param(
        [string]$Method,
        [string]$Path,
        [hashtable]$Headers,
        $Body
    )

    $uri = '{0}{1}' -f $BaseUrl.TrimEnd('/'), $Path
    $jsonBody = $null

    if ($null -ne $Body) {
        $jsonBody = $Body | ConvertTo-Json -Depth 100 -Compress
    }

    if ($DryRun -and $Method -ne 'GET') {
        Write-Host "[dry-run] $Method $uri"
        if ($jsonBody) {
            Write-Host $jsonBody
        }
        return $null
    }

    if ($jsonBody) {
        return Invoke-RestMethod -Uri $uri -Method $Method -Headers $Headers -ContentType 'application/json; charset=utf-8' -Body $jsonBody
    }

    return Invoke-RestMethod -Uri $uri -Method $Method -Headers $Headers
}

function Ensure-Array {
    param($Value)

    if ($null -eq $Value) {
        return @()
    }

    return @($Value)
}

function Resolve-ServiceSlug {
    param(
        [string]$ProposalSlug,
        [array]$ExistingServices
    )

    $aliases = @{
        'diseno-web' = @('diseno-web', 'diseno-de-sitios-web')
        'desarrollo-apps' = @('desarrollo-apps', 'desarrollo-de-aplicaciones')
        'agentes-ia' = @('agentes-ia', 'agentes-de-ia')
        'branding' = @('branding', 'identidad-de-marca')
        'ecommerce' = @('ecommerce', 'e-commerce')
    }

    $candidates = [System.Collections.Generic.List[string]]::new()
    if ($aliases.ContainsKey($ProposalSlug)) {
        foreach ($candidate in @($aliases[$ProposalSlug])) {
            $null = $candidates.Add([string]$candidate)
        }
    }
    else {
        $null = $candidates.Add([string]$ProposalSlug)
    }

    foreach ($candidate in $candidates) {
        $match = $ExistingServices |
            Where-Object {
                $_.slug -eq $candidate -and
                $_.PSObject.Properties.Name -contains 'is_active' -and
                $_.is_active -eq $true
            } |
            Select-Object -First 1
        if ($null -ne $match) {
            return $candidate
        }
    }

    foreach ($candidate in $candidates) {
        $match = $ExistingServices |
            Where-Object {
                $_.slug -eq $candidate -and
                $_.PSObject.Properties.Name -contains 'status' -and
                $_.status -eq 'published'
            } |
            Select-Object -First 1
        if ($null -ne $match) {
            return $candidate
        }
    }

    foreach ($candidate in $candidates) {
        $match = $ExistingServices | Where-Object { $_.slug -eq $candidate } | Select-Object -First 1
        if ($null -ne $match) {
            return $candidate
        }
    }

    return $candidates[0]
}

function Resolve-PlanSlug {
    param([string]$ProposalSlug)

    $normalized = $ProposalSlug.Trim().ToLowerInvariant()
    if ($normalized.EndsWith('basico')) { return 'basico' }
    if ($normalized.EndsWith('profesional')) { return 'medio' }
    if ($normalized.EndsWith('medio')) { return 'medio' }
    if ($normalized.EndsWith('avanzado')) { return 'avanzado' }
    if ($normalized.EndsWith('personalizado')) { return 'personalizado' }
    return $normalized
}

function Convert-ProposalFeatures {
    param([array]$Features)

    return @(Ensure-Array $Features | ForEach-Object {
        @{
            texto = [string]$_
            incluido = $true
        }
    })
}

function Convert-PhasePayload {
    param([array]$Phases)

    return @(Ensure-Array $Phases | ForEach-Object {
        @{
            phase_number = [int]$_.phase_number
            title = [string]$_.title
            description = if ($null -eq $_.description) { $null } else { [string]$_.description }
            percentage_of_total = [int]$_.percentage_of_total
            estimated_days = [int]$_.estimated_days
            max_revisions = [int]$_.max_revisions
        }
    })
}

function Convert-ExistingPlan {
    param($Plan)

    return @{
        id = [string]$Plan.id
        slug = [string]$Plan.slug
        name = [string]$Plan.name
        price_cents = [int]$Plan.price_cents
        description = if ($null -eq $Plan.description) { $null } else { [string]$Plan.description }
        features = @(Ensure-Array $Plan.features | ForEach-Object {
            @{
                texto = [string]$_.texto
                incluido = [bool]$_.incluido
            }
        })
        is_highlighted = [bool]$Plan.is_highlighted
        is_custom = [bool]$Plan.is_custom
        sort_order = [int]$Plan.sort_order
        phases = @(Ensure-Array $Plan.phases | ForEach-Object {
            @{
                phase_number = [int]$_.phase_number
                title = [string]$_.title
                description = if ($null -eq $_.description) { $null } else { [string]$_.description }
                percentage_of_total = [int]$_.percentage_of_total
                estimated_days = [int]$_.estimated_days
                max_revisions = [int]$_.max_revisions
            }
        })
    }
}

function Convert-ProposalPlan {
    param(
        $Plan,
        $ExistingPlan
    )

    $canonicalSlug = Resolve-PlanSlug $Plan.slug
    $payload = @{
        slug = $canonicalSlug
        name = [string]$Plan.name
        price_cents = [int]$Plan.price * 100
        description = if ($null -eq $Plan.description) { $null } else { [string]$Plan.description }
        features = Convert-ProposalFeatures (Ensure-Array $Plan.features)
        is_highlighted = [bool]$Plan.is_highlighted
        is_custom = [bool]$Plan.is_custom
        sort_order = [int]$Plan.sort_order
        phases = Convert-PhasePayload (Ensure-Array $Plan.phases)
    }

    if ($null -ne $ExistingPlan) {
        $payload.id = [string]$ExistingPlan.id
    }

    return $payload
}

if (-not [System.IO.Path]::IsPathRooted($ProposalPath)) {
    $ProposalPath = Join-Path $repoRoot $ProposalPath
}

Write-Host "Cargando propuesta desde $ProposalPath"
$proposal = Get-Content -LiteralPath $ProposalPath -Raw -Encoding UTF8 | ConvertFrom-Json
$token = New-AdminJwt -Secret $JwtSecret -Subject $UserId
$headers = @{ Authorization = "Bearer $token" }

$existingServices = Ensure-Array (Invoke-CmsJson -Method 'GET' -Path '/api/admin/services' -Headers $headers -Body $null)

$results = @()

foreach ($service in Ensure-Array $proposal.services) {
    $targetSlug = Resolve-ServiceSlug -ProposalSlug ([string]$service.slug) -ExistingServices $existingServices
    $currentService = $existingServices | Where-Object { $_.slug -eq $targetSlug } | Select-Object -First 1
    $proposalGallery = @(Ensure-Array $service.gallery)
    $currentGallery = if ($null -ne $currentService) { @(Ensure-Array $currentService.gallery) } else { @() }

    $serviceBody = @{
        title = [string]$service.title
        slug = $targetSlug
        description = if ($null -eq $service.description) { $null } else { [string]$service.description }
        base_price_cents = [int]$service.base_price * 100
        currency = [string]$service.currency
        is_active = [bool]$service.is_active
        image_url = if ($null -ne $service.image_url) { $service.image_url } elseif ($null -ne $currentService) { $currentService.image_url } else { $null }
        gallery = if ($proposalGallery.Count -gt 0) { $proposalGallery } else { $currentGallery }
        skills = @(Ensure-Array $service.skills | ForEach-Object {
            @{
                titulo = [string]$_.titulo
                descripcion = if ($null -eq $_.descripcion) { $null } else { [string]$_.descripcion }
            }
        })
        content = if ($null -eq $service.content) { $null } else { [string]$service.content }
        meta_title = if ($null -eq $service.meta_title) { $null } else { [string]$service.meta_title }
        meta_description = if ($null -eq $service.meta_description) { $null } else { [string]$service.meta_description }
        status = [string]$service.status
        sort_order = [int]$service.sort_order
    }

    if ($null -eq $serviceBody.image_url) {
        $serviceBody.Remove('image_url')
    }

    if (@($serviceBody.gallery).Count -eq 0) {
        $serviceBody.Remove('gallery')
    }

    if ($null -ne $currentService) {
        Write-Host "Actualizando servicio $targetSlug"
        $savedService = Invoke-CmsJson -Method 'PUT' -Path ("/api/admin/services/{0}" -f $currentService.id) -Headers $headers -Body $serviceBody
    }
    else {
        Write-Host "Creando servicio $targetSlug"
        $createBody = @{}
        foreach ($entry in $serviceBody.GetEnumerator()) {
            if ($entry.Key -ne 'is_active') {
                $createBody[$entry.Key] = $entry.Value
            }
        }
        $savedService = Invoke-CmsJson -Method 'POST' -Path '/api/admin/services' -Headers $headers -Body $createBody
    }

    if ($null -eq $savedService) {
        $serviceId = if ($null -ne $currentService) { [string]$currentService.id } else { '<dry-run>' }
        $currentPlans = if ($null -ne $currentService) { Ensure-Array $currentService.plans } else { @() }
    }
    else {
        $serviceId = [string]$savedService.id
        $currentPlans = Ensure-Array $savedService.plans
    }

    $planPayload = @()
    foreach ($plan in Ensure-Array $service.plans) {
        $canonicalPlanSlug = Resolve-PlanSlug ([string]$plan.slug)
        $existingPlan = $currentPlans | Where-Object { $_.slug -eq $canonicalPlanSlug } | Select-Object -First 1
        $planPayload += Convert-ProposalPlan -Plan $plan -ExistingPlan $existingPlan
    }

    if ($serviceId -ne '<dry-run>') {
        try {
            Write-Host "Guardando planes de $targetSlug"
            $null = Invoke-CmsJson -Method 'PUT' -Path ("/api/admin/services/{0}/plans" -f $serviceId) -Headers $headers -Body @{ plans = $planPayload }
        }
        catch {
            $extraPlans = @(Ensure-Array $currentPlans | Where-Object { $_.slug -notin @('basico', 'medio', 'avanzado') } | ForEach-Object { Convert-ExistingPlan $_ })
            if (@($extraPlans).Count -eq 0) {
                throw
            }

            Write-Warning "El guardado de 3 planes fallo para $targetSlug. Reintentando preservando planes legacy extra."
            $fallbackPayload = @($planPayload + $extraPlans)
            $null = Invoke-CmsJson -Method 'PUT' -Path ("/api/admin/services/{0}/plans" -f $serviceId) -Headers $headers -Body @{ plans = $fallbackPayload }
        }
    }
    else {
        Write-Host "[dry-run] Guardando planes de $targetSlug"
    }

    $results += [pscustomobject]@{
        slug = $targetSlug
        service_id = $serviceId
        created = $null -eq $currentService
        plans_requested = @(Ensure-Array $service.plans).Count
    }

    if ($null -eq $currentService -and $serviceId -ne '<dry-run>' -and $null -ne $savedService) {
        $existingServices = @($existingServices + $savedService)
    }
}

Write-Host ''
Write-Host 'Resumen:'
$results | Format-Table -AutoSize