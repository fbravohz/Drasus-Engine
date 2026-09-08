// Renderiza index.html a PDF (2 páginas A4) y a PNG de alta resolución por página.
// Uso: node render.mjs

import puppeteer from "puppeteer";
import path from "node:path";
import { fileURLToPath } from "node:url";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const HTML_PATH = path.join(DIR, "index.html");
const OUT_DIR = path.join(DIR, "output");

const PNG_SCALE = 3; // ~288dpi equivalente para impresión nítida

const browser = await puppeteer.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.goto("file://" + HTML_PATH, { waitUntil: "networkidle0" });

  // --- PDF de 2 páginas ---
  await page.pdf({
    path: path.join(OUT_DIR, "drasus-brochure.pdf"),
    printBackground: true,
    preferCSSPageSize: true,
  });

  // --- PNG por página, a resolución de impresión ---
  await page.setViewport({ width: 794, height: 1123, deviceScaleFactor: PNG_SCALE });
  const pages = await page.$$(".page");
  for (let i = 0; i < pages.length; i++) {
    await pages[i].screenshot({
      path: path.join(OUT_DIR, `drasus-brochure-p${i + 1}.png`),
    });
  }

  console.log(`Listo: ${pages.length} página(s) exportadas a PDF y PNG en ${OUT_DIR}`);
} finally {
  await browser.close();
}
