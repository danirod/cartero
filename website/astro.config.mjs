// SPDX-License-Identifier: GPL-3.0-or-later OR CC-BY-SA-4.0
// SPDX-FileCopyrightText: 2024-2025 the Cartero authors

// @ts-check

import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
  site: "https://cartero.danirod.es",
  integrations: [mdx(), sitemap()],
});
