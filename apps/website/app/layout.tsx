import type { Metadata } from "next";
import "./globals.css";
import "./motion-tour.css";

export const metadata: Metadata = {
  metadataBase: new URL(
    "https://digital-discipline-windows.codecut-official.chatgpt.site",
  ),
  alternates: { canonical: "/" },
  title: "Digital Discipline — Your time. Back in your hands.",
  description:
    "Understand your day, set browser boundaries, and make room for what matters. Digital Discipline is a local-first Windows app for a more intentional digital life.",
  openGraph: {
    title: "Digital Discipline — Your time. Back in your hands.",
    description:
      "Less autopilot. More of your day. Download the local-first Windows app.",
    type: "website",
    images: [
      {
        url: "/social.png",
        width: 1200,
        height: 630,
        alt: "Digital Discipline — Your time, back in your hands",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Digital Discipline",
    description: "Your time. Back in your hands.",
    images: ["/social.png"],
  },
  icons: {
    icon: "/favicon.svg",
    shortcut: "/favicon.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">{children}</body>
    </html>
  );
}
