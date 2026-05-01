"use client";

import { Transcript, TranscriptSegmentData } from '@/types';
import { VirtualizedTranscriptView } from '@/components/VirtualizedTranscriptView';
import { TranscriptButtonGroup } from './TranscriptButtonGroup';
import { useMemo } from 'react';

interface TranscriptPanelProps {
  meetingTitle?: string;
  meetingCreatedAt?: string;
  transcripts: Transcript[];
  onCopyTranscript: () => void;
  onOpenMeetingFolder: () => Promise<void>;
  isRecording: boolean;
  disableAutoScroll?: boolean;

  // Optional pagination props (when using virtualization)
  usePagination?: boolean;
  segments?: TranscriptSegmentData[];
  hasMore?: boolean;
  isLoadingMore?: boolean;
  totalCount?: number;
  loadedCount?: number;
  onLoadMore?: () => void;

  // Retranscription props
  meetingId?: string;
  meetingFolderPath?: string | null;
  onRefetchTranscripts?: () => Promise<void>;
}

export function TranscriptPanel({
  meetingTitle,
  meetingCreatedAt,
  transcripts,
  onCopyTranscript,
  onOpenMeetingFolder,
  isRecording,
  disableAutoScroll = false,
  usePagination = false,
  segments,
  hasMore,
  isLoadingMore,
  totalCount,
  loadedCount,
  onLoadMore,
  meetingId,
  meetingFolderPath,
  onRefetchTranscripts,
}: TranscriptPanelProps) {
  const convertedSegments = useMemo(() => {
    if (usePagination && segments) {
      return segments;
    }

    return transcripts.map(t => ({
      id: t.id,
      timestamp: t.audio_start_time ?? 0,
      endTime: t.audio_end_time,
      text: t.text,
      confidence: t.confidence,
    }));
  }, [transcripts, usePagination, segments]);

  const transcriptCount = usePagination ? (totalCount ?? convertedSegments.length) : (transcripts?.length || 0);

  return (
    <div className="flex w-full min-w-0 bg-white flex-col relative">
      <div className="px-6 py-4 border-b border-gray-200">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            <h1 className="text-xl font-semibold text-gray-900 truncate">
              {meetingTitle || 'Transcript'}
            </h1>
            <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-gray-500">
              <span>{transcriptCount} transcript segment{transcriptCount === 1 ? '' : 's'}</span>
              {meetingCreatedAt && (
                <>
                  <span aria-hidden="true">-</span>
                  <span>{new Date(meetingCreatedAt).toLocaleDateString()}</span>
                </>
              )}
            </div>
          </div>

          <TranscriptButtonGroup
            transcriptCount={transcriptCount}
            onCopyTranscript={onCopyTranscript}
            onOpenMeetingFolder={onOpenMeetingFolder}
            meetingId={meetingId}
            meetingFolderPath={meetingFolderPath}
            onRefetchTranscripts={onRefetchTranscripts}
          />
        </div>
      </div>

      <div className="flex-1 overflow-hidden pb-4">
        <VirtualizedTranscriptView
          segments={convertedSegments}
          isRecording={isRecording}
          isPaused={false}
          isProcessing={false}
          isStopping={false}
          enableStreaming={false}
          showConfidence={true}
          disableAutoScroll={disableAutoScroll}
          hasMore={hasMore}
          isLoadingMore={isLoadingMore}
          totalCount={totalCount}
          loadedCount={loadedCount}
          onLoadMore={onLoadMore}
        />
      </div>
    </div>
  );
}
