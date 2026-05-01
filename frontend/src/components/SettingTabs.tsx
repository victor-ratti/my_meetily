import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { TranscriptModelProps, TranscriptSettings } from "./TranscriptSettings"
import { RecordingSettings } from "./RecordingSettings"
import { About } from "./About";

interface SettingTabsProps {
  transcriptModelConfig: TranscriptModelProps;
  setTranscriptModelConfig: (config: TranscriptModelProps) => void;
  setSaveSuccess: (success: boolean | null) => void;
  defaultTab?: string;
}

export function SettingTabs({
  setSaveSuccess,
  defaultTab = "transcriptSettings",
  transcriptModelConfig,
  setTranscriptModelConfig,
}: SettingTabsProps) {
  const handleTabChange = () => {
    setSaveSuccess(null);
  };

  return (
    <Tabs defaultValue={defaultTab} className="w-full max-h-[calc(100vh-10rem)] overflow-y-auto" onValueChange={handleTabChange}>
      <TabsList>
        <TabsTrigger value="transcriptSettings">Transcript</TabsTrigger>
        <TabsTrigger value="recordingSettings">Preferences</TabsTrigger>
        <TabsTrigger value="about">About</TabsTrigger>
      </TabsList>
      <TabsContent value="transcriptSettings">
        <TranscriptSettings
          transcriptModelConfig={transcriptModelConfig}
          setTranscriptModelConfig={setTranscriptModelConfig}
        />
      </TabsContent>
      <TabsContent value="recordingSettings">
        <RecordingSettings />
      </TabsContent>
      <TabsContent value="about">
        <About />
      </TabsContent>
    </Tabs>
  )
}
