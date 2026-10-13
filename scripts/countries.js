import fs from "node:fs";
import { ulid } from "ulid";

import country from "./seed/countries.json" with {type: "json"};

let inserts = [];

country.forEach((c) => {
    const sql = `(
    '${ulid()}',
    '${c.code.replace(/'/g, "''")}',
    '${c.name.replace(/'/g, "''")}', 
    '${c.country.replace(/'/g, "''")}',
    '${c.flag}'
  )`;
    inserts.push(sql);
});

const fullSQL = `
-- Create table
CREATE TABLE IF NOT EXISTS countries (
  identifier CHAR(26) PRIMARY KEY,
  currency_code VARCHAR(10) NOT NULL,
  currency VARCHAR(100) NOT NULL,
  country VARCHAR(100) NOT NULL,
  flag TEXT
);

-- Insert seed data
INSERT INTO countries (
  identifier,
  currency_code,
  currency,
  country,
  flag
) VALUES
${inserts.join(",\n")}
;
`;

// Write to file
fs.writeFileSync("seed/countries.sql", fullSQL, "utf8");

console.log("✅ Seeder generated: seed_countries.sql");