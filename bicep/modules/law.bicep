param nameFormat string
param location string
param tags object

/*
  Log Analytics Workspace
*/

resource logAnalyticsWorkspace 'Microsoft.OperationalInsights/workspaces@2022-10-01' = {
  name: format(nameFormat, 'LAW', 1)
  location: location
  tags: tags
  properties: {
    sku: {
      name: 'PerGB2018'
    }
    features: {
      enableLogAccessUsingOnlyResourcePermissions: true
    }
    workspaceCapping: {
      dailyQuotaGb: json('0.025')
    }
  }
}
