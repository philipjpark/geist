import "./globals.css";
import type { Metadata } from "next";
import { Providers } from "@/components/Providers";

export const metadata: Metadata = {
  title: "Geist — From Social Signal to Execution Intelligence",
  description: "Semantic execution compiler for fragmented markets — social corpus to execution graph.",
  icons: {
    icon: [{ url: "/geist_logo1.png", type: "image/png" }],
    apple: "/geist_logo1.png",
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
