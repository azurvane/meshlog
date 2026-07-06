interface HomeProps {
  filePath: string;
  onResetPath: () => void; // Add the function to the interface
}

export function Home({ filePath, onResetPath }: HomeProps) {
  return (
    <div style={{ padding: "20px" }}>
      <p style={{ color: "#FFFFFF", marginBottom: "15px" }}>
        <strong>Current Workspace:</strong> {filePath}
      </p>

      <button
        onClick={onResetPath}
        style={{
          backgroundColor: "#ff4d4f",
          color: "white",
          border: "none",
          padding: "8px 16px",
          borderRadius: "4px",
          cursor: "pointer",
          fontWeight: "bold",
        }}
      >
        Reset Project Path
      </button>
    </div>
  );
}
