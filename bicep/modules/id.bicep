param nameFormat string
param location string
param tags object

resource managedIdentityNone 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = {
  name: format(nameFormat, 'ID', 1)
  location: location
  tags: tags
}

resource managedIdentityGet 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = {
  name: format(nameFormat, 'ID', 2)
  location: location
  tags: tags
}

resource managedIdentityList 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = {
  name: format(nameFormat, 'ID', 3)
  location: location
  tags: tags
}

resource managedIdentityGetList 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = {
  name: format(nameFormat, 'ID', 4)
  location: location
  tags: tags
}

output getPrincipalIds array = [
  managedIdentityGet.properties.principalId
  managedIdentityGetList.properties.principalId
]
output listPrincipalIds array = [
  managedIdentityList.properties.principalId
  managedIdentityGetList.properties.principalId
]
