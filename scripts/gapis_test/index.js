const fs = require('fs').promises;
const path = require('path');
const { google } = require("googleapis");
const {authenticate} = require('@google-cloud/local-auth');
const { program } = require("commander");
const { promisify } = require("util");
const { readFile } = require("fs");
const spreadsheetId = "1O9ycQZau4Hy80BXmZQGKmf5CB5fqzRqB3IgHHV1wGgQ";

const SCOPES = ["https://www.googleapis.com/auth/spreadsheets"];
const TOKEN_PATH = path.join(process.cwd(), 'token.json');
const CREDENTIALS_PATH = path.join(process.cwd(), 'credentials.json');

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
    type: 'authorized_user',
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

async function appendValues(auth, values) {
  const sheets = google.sheets({ version: "v4", auth });
  const request = {
    spreadsheetId,
    range: "2023!A:B",
    valueInputOption: "USER_ENTERED",
    resource: {
      values: [values],
    },
    insertDataOption: "INSERT_ROWS",
  };
  await sheets.spreadsheets.values.append(request);
}

async function run() {
  program
    .option("-d, --date <date>", "Date to append in YYYY-MM-DD format")
    .option("-a, --amount <amount>", "Amount to append")
    .parse(process.argv);

  const { date, amount } = program.opts();
  if (!date || !amount) {
    program.help();
    return;
  }

  const auth = await authorize();
  await appendValues(auth, [date, amount]);

  console.log(`Successfully appended values: ${date}, ${amount}`);
}

run().catch(console.error);
