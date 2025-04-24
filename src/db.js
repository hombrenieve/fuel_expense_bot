const { google } = require("googleapis");
const { authenticate } = require("@google-cloud/local-auth");
const path = require("path");
const fs = require("fs").promises;

const SCOPES = ["https://www.googleapis.com/auth/spreadsheets"]; // Google Sheets API scope
const TOKEN_PATH = path.join(process.cwd(), "token.json");
const CREDENTIALS_PATH = path.join(process.cwd(), "credentials.json");

async function loadSavedCredentialsIfExist() {
    try {
        const content = await fs.readFile(TOKEN_PATH);
        const credentials = JSON.parse(content);
        return google.auth.fromJSON(credentials);
    } catch (err) {
        return null;
    }
}

async function saveCredentials(client) {
    const content = await fs.readFile(CREDENTIALS_PATH);
    const keys = JSON.parse(content);
    const key = keys.installed || keys.web;
    const payload = JSON.stringify({
        type: "authorized_user",
        client_id: key.client_id,
        client_secret: key.client_secret,
        refresh_token: client.credentials.refresh_token,
    });
    await fs.writeFile(TOKEN_PATH, payload);
}

async function authorize() {
    let client = await loadSavedCredentialsIfExist();
    if (client) {
        return client;
    }
    client = await authenticate({
        scopes: SCOPES,
        keyfilePath: CREDENTIALS_PATH,
    });
    if (client.credentials) {
        await saveCredentials(client);
    }
    return client;
}

class Db {
    constructor() {
        this.spreadsheetId = "<your_spreadsheet_id>"; // Replace with your Google Sheet ID
    }

    async addAmount(user, date, amount) {
        const auth = await authorize();
        const sheets = google.sheets({ version: "v4", auth });
        const monthIndex = date.getMonth(); // Get month index (0 for January, 1 for February, etc.)
        const column = String.fromCharCode(65 + monthIndex); // Convert to column letter (A for January, B for February, etc.)
        const range = `Sheet1!${column}1:${column}`; // Assuming all months are in "Sheet1"

        // Get existing data to find the first empty row
        const response = await sheets.spreadsheets.values.get({
            spreadsheetId: this.spreadsheetId,
            range,
        });

        const rows = response.data.values || [];
        const nextRow = rows.length + 1; // First empty row

        // Append the data
        await sheets.spreadsheets.values.update({
            spreadsheetId: this.spreadsheetId,
            range: `${month}!A${nextRow}`,
            valueInputOption: "USER_ENTERED",
            resource: {
                values: [[user, date.toISOString(), amount]],
            },
        });

        return amount; // Return the added amount
    }

    async getAmount(user, date) {
        const auth = await authorize();
        const sheets = google.sheets({ version: "v4", auth });
        const monthIndex = date.getMonth(); // Get month index (0 for January, 1 for February, etc.)
        const column = String.fromCharCode(65 + monthIndex); // Convert to column letter (A for January, B for February, etc.)
        const range = `Sheet1!${column}1:${column}`; // Assuming all months are in "Sheet1"

        // Get existing data
        const response = await sheets.spreadsheets.values.get({
            spreadsheetId: this.spreadsheetId,
            range,
        });

        const rows = response.data.values || [];
        let totalAmount = 0;

        // Calculate the total amount for the user
        if (rows.length >= 10) {
            const row = rows[9]; // Row 10 (0-based index is 9)
            if (row[0] === user) {
            totalAmount = parseFloat(row[2]);
            }
        }

        return totalAmount; // Return the total amount
    }
    
    setLimit(user, date, limit) {
       // just update row 11 of the first 12 columns with the limit
        const monthIndex = date.getMonth(); // Get month index (0 for January, 1 for February, etc.)
        const column = String.fromCharCode(65 + monthIndex); // Convert to column letter (A for January, B for February, etc.)
        const range = `Sheet1!${column}11`; // Assuming all months are in "Sheet1"

        // Update the limit
        await sheets.spreadsheets.values.update({
            spreadsheetId: this.spreadsheetId,
            range,
            valueInputOption: "USER_ENTERED",
            resource: {
                values: [[limit]],
            },
        });

        return limit; // Return the set limit
    }

    close() {
        console.log("No persistent connection to close for Google Sheets.");
    }
}

module.exports.Db = Db;