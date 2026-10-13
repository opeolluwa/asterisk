import { fileURLToPath } from "node:url";
import { createServer } from "node:http";
import { readdirSync } from "node:fs";
import nunjucks from "nunjucks";
import { dirname, join } from "node:path";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const templatesDir = join(scriptDir, "..", "templates");

nunjucks.configure(templatesDir, { autoescape: true });

const PORT = Number(process.env.PORT) || 5858;

const DASHBOARD_LINK = "https://www.asterisk.app/console";

const SAMPLE_DATA = {

  "welcome.html": { name: "Paula", dashboard_link: DASHBOARD_LINK },
  "password_reset.html": {
    name: "Paula",
    reset_link: "https://www.asterisk.app/set-password?token=sample-token-123",
  },
  "invitation.html": {
    signup_link:
      "https://www.asterisk.app/auth/signup?token=sample-jwt-token-123",
    invitee_name: "Ada",
    inviter_name: "Ngozi Mensah",
    role_name: "Super Admin",
    expires_on: "4 October 2026",
  },
  "auto_respond.html": { name: "Paula" },
  "invitation_accepted.html": { dashboard_link: DASHBOARD_LINK },
  "payment_confirmed.html": {
    name: "Paula",
    amount: "₦245,000.00",
    reference: "PAY-2026-0917-A1B2",
    dashboard_link: DASHBOARD_LINK,
  },
  "order_resolved.html": {
    name: "Paula",
    order_number: "ORD-1042",
    dashboard_link: DASHBOARD_LINK,
  },
  "welcome_newsletter.html": { name: "Paula", dashboard_link: DASHBOARD_LINK },
  // The one that carries public, caller-controlled text. The payload here is a
  // benign multi-line message; to confirm the escaping by eye, paste something
  // like `<script>alert(1)</script>` into `body` and reload — it must come out
  // as visible characters, not markup.
  "email_forward.html": {
    sender_name: "Paula Okoye",
    sender_email: "paula.okoye@example.com",
    recipient_email: "support@example.com",
    // Shown only when non-empty, so clear it here to see the row disappear.
    phone: "+234 806 657 0002",
    subject: "Nipple line is leaking overnight",
    body: "Good afternoon,\n\nThe nipple line on the layer 2 pen started dripping around 2am and the bedding was soaked by the time I got out there. I have closed the valve for now.\n\nCould someone come out today? The animals are fine, it is the litter that is a problem.\n\nThanks,\nPaula",
  },
  "otp_code.html": { name: "Paula", code: "654321" },
  "transaction.html": {
    name: "Paula",
    transaction_reference: "ORD-0042",
    total: "₦295,000.00",
    items: [
      { description: "Website design", quantity: "1", amount: "₦180,000.00" },
      { description: "Hosting — annual", quantity: "1", amount: "₦40,000.00" },
      { description: "Maintenance retainer", quantity: "3", amount: "₦25,000.00" },
    ],
  },
};

/**
 * The bare-minimum context, used for any template without a SAMPLE_DATA entry
 * and as the base that per-request query overrides are layered onto. The
 * invitation's optional fields are empty here so `/?` shows the degraded
 * rendering — no inviter, no role, no expiry — which is a case worth eyeballing
 * because it is what a legacy invitation row produces.
 */
const DEFAULT_SAMPLE = {
  name: "Paula",
  dashboard_link: DASHBOARD_LINK,
  reset_link: `${DASHBOARD_LINK}reset-password?token=sample-token-123`,
  code: "123456",
  amount: "₦50,000.00",
  reference: "PAY-2026-0000-0000",
  order_number: "ORD-0000",
  transaction_reference: "ORD-0000",
  total: "₦0.00",
  signup_link: `${DASHBOARD_LINK}auth/signup?token=sample-token-123`,
  invitee_name: "Paula",
  inviter_name: "",
  role_name: "",
  expires_on: "",
  items: [],
};

function slugify(file) {
  return file.replace(/\.html$/, "").replace(/_/g, "-");
}

function labelFor(file) {
  const name = file.replace(/\.html$/, "").replace(/_/g, " ");
  return name.replace(/\b\w/g, (c) => c.toUpperCase());
}

function scanTemplates() {
  return readdirSync(templatesDir)
    .filter((file) => file.endsWith(".html") && file !== "base.html")
    .sort()
    .map((file) => ({ route: slugify(file), file, label: labelFor(file) }));
}

const TEMPLATES = scanTemplates();

function readIndexHtml() {
  const links = TEMPLATES.map(
    ({ route, label }) =>
      `<li style="margin-bottom:10px"><a href="/${route}" style="color:#00b87a;font-weight:600">${label}</a> <span style="color:#8a9490">— /${route}</span></li>`,
  ).join("\n    ");
  return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Email template preview</title>
    <style>
      body {
        font-family: "Poppins", Arial, sans-serif;
        background: #ffffff;
        color: #0d1220;
        margin: 0;
        padding: 40px 24px;
      }
      h1 { font-size: 22px; margin: 0 0 4px; letter-spacing: -0.3px; }
      p.sub { color: #8a9490; font-size: 14px; margin: 0 0 28px; }
      ul { list-style: none; padding: 0; margin: 0 0 32px; }
      .scan {
        display: inline-block;
        background: #00b87a;
        color: #fff;
        text-decoration: none;
        font-weight: 600;
        font-size: 14px;
        padding: 12px 24px;
        border-radius: 999px;
      }
    </style>
  </head>
  <body>
    <h1>Email template preview</h1>
    <p class="sub">Rendered live from asterisk/templates using nunjucks (askama-compatible). Auto-indexed from the templates directory.</p>
    <ul>
      ${links}
    </ul>
    <a class="scan" href="/all">Scan all templates</a>
  </body>
</html>`;
}

function readAllHtml() {
  const frames = TEMPLATES.map(
    ({ route, label }) =>
      `<div style="margin-bottom:24px;border:1px solid #e3e9e6;border-radius:12px;overflow:hidden;background:#fff">
        <div style="padding:10px 16px;background:#f7f9f8;font-size:12px;color:#8a9490;font-weight:600">${label} — /${route}</div>
        <iframe src="/${route}" style="width:100%;height:760px;border:0"></iframe>
      </div>`,
  ).join("\n    ");
  return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>All email templates</title>
    <style>
      body { background: #eef2f0; margin: 0; padding: 24px; }
    </style>
  </head>
  <body>
    ${frames}
  </body>
</html>`;
}

function buildContext(templateFile, url) {
  const overrides = {
    name: url.searchParams.get("name"),
    dashboard_link: url.searchParams.get("dashboardLink"),
    reset_link: url.searchParams.get("resetLink"),
    code: url.searchParams.get("code"),
    amount: url.searchParams.get("amount"),
    reference: url.searchParams.get("reference"),
    order_number: url.searchParams.get("orderNumber"),
    transaction_reference: url.searchParams.get("transactionReference"),
    total: url.searchParams.get("total"),
    signup_link: url.searchParams.get("signupLink"),
    invitee_name: url.searchParams.get("inviteeName"),
    inviter_name: url.searchParams.get("inviterName"),
    role_name: url.searchParams.get("roleName"),
    expires_on: url.searchParams.get("expiresOn"),
  };
  const ctx = { ...DEFAULT_SAMPLE, ...(SAMPLE_DATA[templateFile] ?? {}) };
  for (const [key, value] of Object.entries(overrides)) {
    if (value !== null) ctx[key] = value;
  }
  return ctx;
}

const server = createServer((req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  const pathname = url.pathname;

  if (pathname === "/favicon.ico") {
    res.writeHead(204);
    res.end();
    return;
  }

  if (pathname === "/") {
    res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
    res.end(readIndexHtml());
    return;
  }

  if (pathname === "/all") {
    res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
    res.end(readAllHtml());
    return;
  }

  const template = TEMPLATES.find(({ route }) => route === pathname.slice(1));

  if (template === undefined) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("Not found");
    return;
  }

  const html = nunjucks.render(template.file, buildContext(template.file, url));

  res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
  res.end(html);
});

server.listen(PORT, () => {
  console.log(`Email template preview running at http://localhost:${PORT}`);
  console.log(`Templates directory: ${templatesDir}`);
  console.log(`Press Ctrl+C to stop.`);
});