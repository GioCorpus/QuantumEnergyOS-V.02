// ══════════════════════════════════════════════════════════════════════
//  QuantumEnergyOS — Azure Deployment (Bicep)
//  Despliega en: Azure Container Apps + Azure Quantum + Azure Monitor
//  Regiones: East US, West US 2, Mexico Central, West Europe
// ══════════════════════════════════════════════════════════════════════

@description('Nombre base del despliegue')
param appName string = 'quantumenergyos'

@description('Región Azure — Mexico Central para latencia mínima desde Mexicali')
@allowed([
  'mexicocentral'
  'eastus'
  'westus2'
  'westeurope'
  'japaneast'
])
param location string = 'mexicocentral'

@description('Versión de la imagen Docker')
param imageTag string = 'latest'

@description('SKU del plan de Container Apps')
@allowed(['Consumption', 'Dedicated'])
param containerAppsSku string = 'Consumption'

@secure()
@description('JWT Secret — inyectado desde Key Vault en producción')
param jwtSecret string

@secure()
@description('IBM Quantum Token')
param ibmQuantumToken string = ''

// ── Variables ─────────────────────────────────────────────────
var resourcePrefix = '${appName}-${uniqueString(resourceGroup().id)}'
var containerImage = 'ghcr.io/giocorpus/quantumenergyos:${imageTag}'
var tags = {
  project: 'QuantumEnergyOS'
  origin: 'Mexicali-BC-Mexico'
  mission: 'Kardashev-0-to-1'
  managedBy: 'GioCorpus'
}

// ── Log Analytics Workspace ───────────────────────────────────
resource logAnalytics 'Microsoft.OperationalInsights/workspaces@2023-09-01' = {
  name: '${resourcePrefix}-logs'
  location: location
  tags: tags
  properties: {
    sku: {
      name: 'PerGB2018'
    }
    retentionInDays: 30
    features: {
      enableLogAccessUsingOnlyResourcePermissions: true
    }
  }
}

// ── Container Apps Environment ────────────────────────────────
resource containerAppsEnv 'Microsoft.App/managedEnvironments@2024-03-01' = {
  name: '${resourcePrefix}-env'
  location: location
  tags: tags
  properties: {
    appLogsConfiguration: {
      destination: 'log-analytics'
      logAnalyticsConfiguration: {
        customerId: logAnalytics.properties.customerId
        sharedKey: logAnalytics.listKeys().primarySharedKey
      }
    }
    workloadProfiles: containerAppsSku == 'Dedicated' ? [
      {
        name: 'Dedicated'
        workloadProfileType: 'D4'
        minimumCount: 1
        maximumCount: 3
      }
    ] : []
  }
}

// ── Container App — API Principal ────────────────────────────
resource containerApp 'Microsoft.App/containerApps@2024-03-01' = {
  name: '${appName}-api'
  location: location
  tags: tags
  identity: {
    type: 'SystemAssigned'   // MSI para acceso a Key Vault
  }
  properties: {
    managedEnvironmentId: containerAppsEnv.id
    configuration: {
      // HTTPS obligatorio — no HTTP
      ingress: {
        external: true
        targetPort: 8000
        transport: 'http'
        allowInsecure: false
        traffic: [
          {
            weight: 100
            latestRevision: true
          }
        ]
        corsPolicy: {
          allowedOrigins: ['*']
          allowedMethods: ['GET', 'POST', 'OPTIONS']
          allowedHeaders: ['Authorization', 'Content-Type']
        }
      }
      // Secretos — nunca en texto plano en el template
      secrets: [
        {
          name: 'jwt-secret'
          value: jwtSecret
        }
        {
          name: 'ibm-quantum-token'
          value: ibmQuantumToken
        }
      ]
      registries: [
        {
          server: 'ghcr.io'
          username: 'GioCorpus'
          passwordSecretRef: 'ghcr-token'
        }
      ]
    }
    template: {
      containers: [
        {
          name: 'quantumenergyos'
          image: containerImage
          resources: {
            cpu: json('1.0')
            memory: '2Gi'
          }
          env: [
            {
              name: 'QEOS_ENV'
              value: 'production'
            }
            {
              name: 'QEOS_PORT'
              value: '8000'
            }
            {
              name: 'QEOS_LOG_LEVEL'
              value: 'INFO'
            }
            {
              name: 'QEOS_JWT_SECRET'
              secretRef: 'jwt-secret'
            }
            {
              name: 'IBM_QUANTUM_TOKEN'
              secretRef: 'ibm-quantum-token'
            }
            {
              name: 'AZURE_QUANTUM_LOCATION'
              value: location
            }
          ]
          probes: [
            {
              type: 'Liveness'
              httpGet: {
                path: '/health'
                port: 8000
              }
              initialDelaySeconds: 30
              periodSeconds: 30
              timeoutSeconds: 10
              failureThreshold: 3
            }
            {
              type: 'Readiness'
              httpGet: {
                path: '/health'
                port: 8000
              }
              initialDelaySeconds: 10
              periodSeconds: 10
            }
          ]
        }
      ]
      scale: {
        minReplicas: 1
        maxReplicas: 10
        rules: [
          {
            name: 'http-scaling'
            http: {
              metadata: {
                concurrentRequests: '50'
              }
            }
          }
        ]
      }
    }
  }
}

// ── Azure Quantum Workspace ───────────────────────────────────
resource quantumWorkspace 'Microsoft.Quantum/workspaces@2023-11-13-preview' = {
  name: '${appName}-quantum'
  location: location
  tags: tags
  identity: {
    type: 'SystemAssigned'
  }
  properties: {
    providers: [
      {
        providerId: 'Microsoft'
        providerSku: 'free'
      }
      {
        providerId: 'ionq'
        providerSku: 'pay-as-you-go-credits'
      }
    ]
    storageAccount: storageAccount.id
  }
}

// ── Storage Account para Azure Quantum ────────────────────────
resource storageAccount 'Microsoft.Storage/storageAccounts@2023-05-01' = {
  name: replace('${resourcePrefix}st', '-', '')
  location: location
  tags: tags
  sku: {
    name: 'Standard_LRS'
  }
  kind: 'StorageV2'
  properties: {
    supportsHttpsTrafficOnly: true
    minimumTlsVersion: 'TLS1_2'
    allowBlobPublicAccess: false
    networkAcls: {
      defaultAction: 'Deny'
      bypass: 'AzureServices'
    }
  }
}

// ── Outputs ───────────────────────────────────────────────────
output apiUrl string = 'https://${containerApp.properties.configuration.ingress.fqdn}'
output quantumWorkspaceId string = quantumWorkspace.id
output logAnalyticsId string = logAnalytics.id
output containerAppName string = containerApp.name

// ══════════════════════════════════════════════════════════════
// Deploy:
//   az group create -n qeos-rg -l mexicocentral
//   az deployment group create \
//     -g qeos-rg -f azure.bicep \
//     --parameters jwtSecret=$(openssl rand -hex 32)
// ══════════════════════════════════════════════════════════════
