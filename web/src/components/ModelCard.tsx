import type { ModelInfo } from "../types";

interface ModelCardProps {
    models: ModelInfo[];
    selectedModel: string;
    onModelChange: (model: string) => void;
    onRefresh: () => void;
}

export default function ModelCard({ models, selectedModel, onModelChange, onRefresh }: ModelCardProps) {
    const hasModels = models.length > 0;

    return (
         <div className="card">
              <div className="card-header">
                  <span className="card-icon">{`\u{1F916}`}</span>
                   <div>
                       <div className="card-title">Model Configuration</div>
                       <div className="card-subtitle">
                           {hasModels
                               ? `${models.length} models detected`
                               : "No models detected — start Ollama or check endpoint settings."}
                       </div>
                   </div>
               </div>
               <div className="model-select-wrap">
                   <select
                      className="form-select"
                      value={selectedModel}
                      onChange={(e) => onModelChange(e.target.value)}
                   >
                       <option value="">-- Select a model --</option>
                       {models.map((m) => (
                           <option key={m.name} value={m.name}>{m.name}</option>
                       ))}
                   </select>
                   <button className="btn btn-secondary btn-sm btn-icon" onClick={onRefresh}>
                       Refresh
                   </button>
               </div>
               {selectedModel && (
                   <div style={{ marginTop: 12 }}>
                       <div style={{ fontSize: 12, color: "var(--text-secondary)", marginBottom: 4 }}>Model Details</div>
                       <div style={{ fontSize: 12, color: "var(--text-primary)" }}>
                           {models.find((m) => m.name === selectedModel)?.size != null && (
                               <div>Size: {Math.round((models.find((m) => m.name === selectedModel)!.size || 0) / 1e6)} MB</div>
                           )}
                       </div>
                   </div>
               )}
           </div>
       );
}
