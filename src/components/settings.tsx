import { Button, Flex, Form, Radio, Select, Typography } from "antd";
import { LoadingOutlined, MutedOutlined, SoundOutlined } from "@ant-design/icons";

import {
  FEMALE_VOICES, MALE_VOICES, TTS_MODELS, TtsOption, TtsSettings, withCurrentOption,
} from "../common/settings";
import { TtsVoice } from "../common/speech";

export interface Props {
  tts: TtsSettings;
  // 環境変数を反映した既定値。一覧にない値でも選択肢に残す。
  ttsDefaults: TtsSettings;
  ttsVoice: TtsVoice;
  isTtsCustomized: boolean;
  imageMaxSize: number;
  imageSizeOptions: readonly number[];
  // 試聴中の声と状態。試聴中でなければ null。
  preview: { voice: TtsVoice; state: "loading" | "playing" } | null;
  onTtsChange: (patch: Partial<TtsSettings>) => void;
  onTtsVoiceChange: (voice: TtsVoice) => void;
  onTtsReset: () => void;
  onPreview: (voice: TtsVoice) => void;
  onImageMaxSizeChange: (size: number) => void;
}

export const SettingsComponent = (props: Props) => {
  const voiceRow = (
    gender: TtsVoice, label: string, options: readonly TtsOption[], value: string, defaultValue: string,
  ) => {
    const previewing = props.preview?.voice === gender ? props.preview.state : null;
    return (
      <Flex gap="small" align="center">
        <Radio value={gender} style={{ width: 64 }}>{label}</Radio>
        <Select
          value={value}
          style={{ flex: 1, minWidth: 0 }}
          aria-label={`${label}の声`}
          options={withCurrentOption(options, value, defaultValue)}
          onChange={(voice: string) =>
            props.onTtsChange(gender === "female" ? { femaleVoice: voice } : { maleVoice: voice })}
        />
        <Button
          onClick={() => props.onPreview(gender)}
          aria-label={previewing ? `${label}の声の試聴を停止` : `${label}の声を試聴`}
          aria-pressed={Boolean(previewing)}
          title={previewing ? "試聴を停止" : `${value} を試聴`}
          icon={previewing === "loading" ? <LoadingOutlined />
            : previewing === "playing" ? <MutedOutlined /> : <SoundOutlined />}
        />
      </Flex>
    );
  };

  return (
    <Flex vertical gap="middle" style={{ padding: "0 8px 96px" }}>
      <Form layout="horizontal" labelCol={{ flex: "96px" }} labelAlign="left" colon={false}>
        <Typography.Title level={5}>読み上げ</Typography.Title>
        <Form.Item label="モデル">
          <Select
            value={props.tts.model}
            aria-label="読み上げモデル"
            options={withCurrentOption(TTS_MODELS, props.tts.model, props.ttsDefaults.model)}
            onChange={(model: string) => props.onTtsChange({ model })}
          />
        </Form.Item>
        <Form.Item label="声" extra="選択した声で回答を読み上げます。回答は翻訳せず原文の言語のまま読みます。">
          <Radio.Group
            value={props.ttsVoice}
            onChange={(e) => props.onTtsVoiceChange(e.target.value)}
            style={{ width: "100%" }}
          >
            <Flex vertical gap="small">
              {voiceRow("female", "女性", FEMALE_VOICES, props.tts.femaleVoice, props.ttsDefaults.femaleVoice)}
              {voiceRow("male", "男性", MALE_VOICES, props.tts.maleVoice, props.ttsDefaults.maleVoice)}
            </Flex>
          </Radio.Group>
        </Form.Item>
        <Form.Item label=" " extra="環境変数 GEMINI_TTS_MODEL / GEMINI_TTS_VOICE_FEMALE / GEMINI_TTS_VOICE_MALE があれば、それが既定値になります。">
          <Button onClick={props.onTtsReset} disabled={!props.isTtsCustomized}>既定に戻す</Button>
        </Form.Item>

        <Typography.Title level={5}>画像</Typography.Title>
        <Form.Item label="リサイズ上限" extra="貼り付けた画像を送信前にこのサイズ以内へ縮小します。">
          <Select
            value={props.imageMaxSize}
            style={{ width: 120 }}
            aria-label="画像リサイズ上限"
            options={props.imageSizeOptions.map((size) => ({ label: `${size}px`, value: size }))}
            onChange={props.onImageMaxSizeChange}
          />
        </Form.Item>
      </Form>
    </Flex>
  );
};
