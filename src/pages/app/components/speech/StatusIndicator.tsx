import { AlertCircleIcon, LoaderIcon } from "lucide-react";

type Props = {
  setupRequired: boolean;
  error: string;
  isProcessing: boolean;
  isAIProcessing: boolean;
  capturing: boolean;
  isPaused: boolean;
};

export const StatusIndicator = ({
  setupRequired,
  error,
  isProcessing,
  isAIProcessing,
  capturing,
  isPaused,
}: Props) => {
  // Don't show anything if not capturing and no error
  if (!capturing && !error && !isProcessing && !isAIProcessing) {
    return null;
  }

  return (
    <div className="flex flex-1 items-center gap-2 px-3 py-2 justify-end">
      {/* Priority: Fatal Error > AI Processing > Transcribing > Warning > Paused > Listening */}
      {error && !setupRequired && !capturing ? (
        <div className="flex items-center gap-2 text-red-600">
          <AlertCircleIcon className="w-4 h-4" />
          <span className="text-xs font-medium">{error}</span>
        </div>
      ) : isAIProcessing ? (
        <div className="flex items-center gap-2 animate-pulse">
          <LoaderIcon className="w-4 h-4 animate-spin" />
          <span className="text-xs font-medium">Generating response...</span>
        </div>
      ) : isProcessing ? (
        <div className="flex items-center gap-2 animate-pulse">
          <LoaderIcon className="w-4 h-4 animate-spin" />
          <span className="text-xs font-medium">Transcribing...</span>
        </div>
      ) : error && !setupRequired && capturing ? (
        <div className="flex items-center gap-2 text-amber-600">
          <AlertCircleIcon className="w-4 h-4" />
          <span className="text-xs font-medium">{error}</span>
        </div>
      ) : capturing && isPaused ? (
        <div className="flex items-center gap-2 text-amber-600">
          <div className="w-2 h-2 rounded-full bg-amber-500" />
          <span className="text-xs font-medium">Paused</span>
        </div>
      ) : capturing ? (
        <div className="flex items-center gap-2 text-green-600 animate-pulse">
          <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
          <span className="text-xs font-medium">Listening...</span>
        </div>
      ) : null}
    </div>
  );
};
