targetScope = 'subscription'

/*
  Parameters
*/

@allowed([
  'D' // Development
  'T' // Test
  'A' // Acceptance
  'P' // Production
])
param environment string
param location string = 'westeurope'
param name object = {
  tenantId: 'BVDB'
  projectId: 'KEYWEAVE'
  region: 'WEU'
}

/*
  Variables
*/

var tags = {
  project: 'keyweave'
}
var nameFormat = '${name.tenantId}-${name.projectId}-${environment}-${name.region}-{0}-{1:N0}'

/*
  Resource Group
*/

resource ResourceGroup 'Microsoft.Resources/resourceGroups@2022-09-01' = {
  name: format(nameFormat, 'RG', 1)
  location: location
  tags: tags
}

/*
  Module for Log Analytics Workspace
*/

module LogAnalyticsWorkspace 'modules/law.bicep' = {
  name: 'LogAnalyticsWorkspace'
  scope: ResourceGroup
  params: {
    nameFormat: nameFormat
    location: location
    tags: tags
  }
}

/*
  Module for Managed Identities
*/

module ManagedIdentities 'modules/id.bicep' = {
  name: 'ManagedIdentities'
  scope: ResourceGroup
  params: {
    nameFormat: nameFormat
    location: location
    tags: tags
  }
}

/*
  Module for KeyVault
*/

module KeyVault 'modules/kv.bicep' = {
  name: 'KeyVault'
  scope: ResourceGroup
  dependsOn: [
    LogAnalyticsWorkspace
  ]
  params: {
    nameFormat: nameFormat
    location: location
    tags: tags

    getPrincipalIds: ManagedIdentities.outputs.getPrincipalIds
    listPrincipalIds: ManagedIdentities.outputs.listPrincipalIds
  }
}
