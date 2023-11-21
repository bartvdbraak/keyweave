param nameFormat string
param location string
param tags object

param getPrincipalIds array
param listPrincipalIds array

var accessPolicies = [for id in union(getPrincipalIds, listPrincipalIds): {
  tenantId: tenant().tenantId
  objectId: id
  permissions: {
    secrets: contains(getPrincipalIds, id) && contains(listPrincipalIds, id) ? ['Get', 'List'] : contains(listPrincipalIds, id) ? ['List'] : ['Get']
  }
}]

/*
  Log Analytics Workspace (existing)
*/

resource _logAnalyticsWorkspace 'Microsoft.OperationalInsights/workspaces@2022-10-01' existing = {
  name: format(nameFormat, 'LAW', 1)
}

/*
  Key Vault
*/

resource keyVault 'Microsoft.KeyVault/vaults@2023-02-01' = {
  name: replace(toLower(format(nameFormat, 'KVT', 1)), '-', '')
  location: location
  tags: tags
  properties: {
    sku: {
      family: 'A'
      name: 'standard'
    }
    tenantId: tenant().tenantId
    enableSoftDelete: true
    enablePurgeProtection: true
    accessPolicies: accessPolicies
  }
  resource testSecret 'secrets' = {
    name: 'testSecret'
    properties: {
      value: 'testSecretValue'
    }
  }
  resource filterTestSecret 'secrets' = {
    name: 'filterTestSecret'
    properties: {
      value: 'filterTestSecretValue'
    }
  }
}

/*
  Diagnostic Settings for Key Vault
*/

resource keyVaultDiagnosticSettings 'Microsoft.Insights/diagnosticSettings@2021-05-01-preview' = {
  name: 'keyVaultLogging'
  scope: keyVault
  properties: {
    workspaceId: _logAnalyticsWorkspace.id
    logs: [
      {
        category: 'AuditEvent'
        enabled: true
      }
    ]
  }
}