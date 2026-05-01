import { useCallback } from 'react';
import { Transcript } from '@/types';
import { toast } from 'sonner';
import Analytics from '@/lib/analytics';
import { invoke as invokeTauri } from '@tauri-apps/api/core';

interface UseCopyOperationsProps {
  meeting: any;
  meetingTitle: string;
}

export function useCopyOperations({
  meeting,
  meetingTitle,
}: UseCopyOperationsProps) {
  const fetchAllTranscripts = useCallback(async (meetingId: string): Promise<Transcript[]> => {
    try {
      console.log('Fetching all transcripts for copying:', meetingId);

      const firstPage = await invokeTauri('api_get_meeting_transcripts', {
        meetingId,
        limit: 1,
        offset: 0,
      }) as { transcripts: Transcript[]; total_count: number; has_more: boolean };

      const totalCount = firstPage.total_count;
      if (totalCount === 0) {
        return [];
      }

      const allData = await invokeTauri('api_get_meeting_transcripts', {
        meetingId,
        limit: totalCount,
        offset: 0,
      }) as { transcripts: Transcript[]; total_count: number; has_more: boolean };

      return allData.transcripts;
    } catch (error) {
      console.error('Error fetching all transcripts:', error);
      toast.error('Failed to fetch transcripts for copying');
      return [];
    }
  }, []);

  const handleCopyTranscript = useCallback(async () => {
    const allTranscripts = await fetchAllTranscripts(meeting.id);

    if (!allTranscripts.length) {
      toast.error('No transcripts available to copy');
      return;
    }

    const formatTime = (seconds: number | undefined, fallbackTimestamp: string): string => {
      if (seconds === undefined) {
        return fallbackTimestamp;
      }

      const totalSecs = Math.floor(seconds);
      const mins = Math.floor(totalSecs / 60);
      const secs = totalSecs % 60;
      return `[${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}]`;
    };

    const header = `# Transcript of the Meeting: ${meeting.id} - ${meetingTitle ?? meeting.title}\n\n`;
    const date = `## Date: ${new Date(meeting.created_at).toLocaleDateString()}\n\n`;
    const fullTranscript = allTranscripts
      .map(t => `${formatTime(t.audio_start_time, t.timestamp)} ${t.text}  `)
      .join('\n');

    await navigator.clipboard.writeText(header + date + fullTranscript);
    toast.success('Transcript copied to clipboard');

    const wordCount = allTranscripts
      .map(t => t.text.split(/\s+/).length)
      .reduce((a, b) => a + b, 0);

    await Analytics.trackCopy('transcript', {
      meeting_id: meeting.id,
      transcript_length: allTranscripts.length.toString(),
      word_count: wordCount.toString(),
    });
  }, [meeting, meetingTitle, fetchAllTranscripts]);

  return {
    handleCopyTranscript,
  };
}
