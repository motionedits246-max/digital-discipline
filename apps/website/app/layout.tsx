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
    "Understand your day and set boundaries with Digital Discipline. Download the local-first Windows app or the Android APK.",
  openGraph: {
    title: "Digital Discipline — Your time. Back in your hands.",
    description:
      "Download Digital Discipline for Windows and Android.",
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
