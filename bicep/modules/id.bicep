param nameFormat string
param location string
param tags object

param identityEnvironments array = [
  'none'
  'get'
  'list'
  'getlist'
]

resource managedIdentity 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = [for (environment, index) in identityEnvironments: {
  name: format(nameFormat, 'ID', index+1)
  location: location
  tags: tags
}]

resource federatedCredential 'Microsoft.ManagedIdentity/userAssignedIdentities/federatedIdentityCredentials@2023-01-31' = [for (environment, index) in identityEnvironments: {
  name: environment
  parent: managedIdentity[index+1]
  properties: {
    issuer: 'https://token.actions.githubusercontent.com'
    subject: 'repo:bartvdbraak/keyweave:environment:${environment}'
    audiences: [
      'api://AzureADTokenExchange'
    ]
  }
}]

output identities array = [for (environment, index) in identityEnvironments: {
  name: environment
  id: managedIdentity[index+1].properties.principalId
}]
