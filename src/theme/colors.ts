export const colors = {
  bgMain: "#0B0D11",       // Darkest background of the app
  bgCard: "#11141A",       // Main panel background color
  bgFooter: "#0E1116",     // Setup footer background color
  border: "#1E232D",       // Panel dividing line color
  textMuted: "#6B7584",    // Muted slate gray text
  textBright: "#ECEFF4",   // Crisp white/silver text
  textDark: "#05070a",     // Dark text for contrast (e.g. on cyan button)
  accentCyan: "#00A3C4",   // The vibrant cyan used for terminals/buttons
  accentCyanHover: "#00b4d4", // The bright cyan highlight state on hover
};

// Inject colors as CSS custom properties on the document root
if (typeof document !== "undefined") {
  const root = document.documentElement;
  Object.entries(colors).forEach(([key, value]) => {
    root.style.setProperty(`--color-${key}`, value);
  });
}