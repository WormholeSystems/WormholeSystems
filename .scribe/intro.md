# Introduction

RESTful API for managing wormhole maps and solar system data in EVE Online. Access and manipulate collaborative mapping data for navigating wormhole space.

<aside>
    <strong>Base URL</strong>: <code>https://wormhole.systems</code>
</aside>

 Welcome to the Tunnel Vision API! This API provides programmatic access to wormhole mapping functionality for EVE Online players and corporations.

 ## What You Can Do

 The API allows you to:
    - **Retrieve maps**: Get your accessible wormhole maps and their details
    - **Manage solar systems**: Add, update, and remove solar systems from your maps
    - **Access collaborative data**: Work with shared mapping intel from your corporation or alliance

## Getting Started

All API endpoints require authentication using a personal access token. You can generate your API token by:

1. Logging into the application
2. Navigate to your account settings
3. Go to the "API Tokens" section
4. Click "Create New Token" and give it a descriptive name
5. Copy the generated token (you won't be able to see it again!)

Include your token in requests using the `Authorization: Bearer {your-token}` header.

