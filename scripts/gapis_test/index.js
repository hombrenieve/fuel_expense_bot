const fs = require('fs').promises;
const path = require('path');
const { google } = require("googleapis");
const { Utilities } = require('googleapis/build/src/apis/script');
const {authenticate} = require('@google-cloud/local-auth');
const { program } = require("commander");
const spreadsheetId = "1O9ycQZau4Hy80BXmZQGKmf5CB5fqzRqB3IgHHV1wGgQ";
const sheetId = "2023";

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

async function findMonthRange(auth, currentDate) {
  const sheets = google.sheets({ version: "v4", auth });
  

  const sheet = await sheets.spreadsheets.get({
    spreadsheetId,
    ranges: [sheetId],
    includeGridData: true,
  });
  const rows = sheet.data.sheets[0].data[0].rowData;
  console.log(sheet.data.properties.locale);

  const firstDayOfMonth = new Date(currentDate.getFullYear(), currentDate.getMonth(), 1);
  const lastDayOfMonth = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 0);

  let startRow = -1;
  let endRow = -1;
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    if (!row.values) {
      continue;
    }

    const dateString = row.values[0].formattedValue;
    if (!dateString) {
      continue;
    }

    const date = new Date(dateString);
    if (date >= firstDayOfMonth && date <= lastDayOfMonth) {
      if (startRow === -1) {
        startRow = i + 1;
      }
      endRow = i + 1;
    }
  }

  return startRow === -1 ? null : `${sheetId}!A${startRow}:B${endRow}`;
}


async function appendValues(auth, range, values) {
  const sheets = google.sheets({ version: "v4", auth });
  const request = {
    spreadsheetId,
    range: range,
    valueInputOption: "USER_ENTERED",
    resource: {
      values: [values],
    },
    insertDataOption: "INSERT_ROWS",
  };
  await sheets.spreadsheets.values.append(request);
}

async function removeRow(auth, row) {
  const sheets = google.sheets({ version: "v4", auth });
  const request = {
    spreadsheetId,
    resource: {
      requests: [
        {
          deleteDimension: {
            range: {
              sheetId: await getSheetId(sheets),
              dimension: 'ROWS',
              startIndex: row - 1,
              endIndex: row,
            },
          },
        },
      ],
    },
  };

  await sheets.spreadsheets.batchUpdate(request);
}

async function getSheetId(sheets) {
  const response = await sheets.spreadsheets.get({ spreadsheetId });
  const sheet = response.data.sheets.find((s) => s.properties.title === `${sheetId}`);
  return sheet.properties.sheetId;
}

async function insertValues(auth, range, values) {
  await appendValues(auth, range, values);
  await removeRow(auth, 13);
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
  await insertValues(auth, `${sheetId}!A1:B12`, [date, amount]);

  console.log(`Successfully appended values: ${date}, ${amount}`);
}

run().catch(console.error);
//For testing individual functions, comment previous line and uncomment the following
//module.exports = { authorize, findMonthRange }